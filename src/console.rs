use std::convert::TryInto;
use std::io;
use std::os::windows::io::AsHandle;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::BorrowedHandle;
use std::ptr;

use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Console::GetConsoleMode;
use windows_sys::Win32::System::Console::WriteConsoleW;

const TRUE: BOOL = 1;

fn check_syscall(result: BOOL) -> io::Result<()> {
    if result == TRUE {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn raw_handle(handle: BorrowedHandle<'_>) -> HANDLE {
    handle.as_raw_handle() as _
}

#[derive(Clone, Copy)]
pub struct Console<'a>(BorrowedHandle<'a>);

impl<'a> Console<'a> {
    pub(super) fn from_handle<T>(handle: &'a T) -> Option<Self>
    where
        T: AsHandle + ?Sized,
    {
        let handle = handle.as_handle();
        let raw_handle = raw_handle(handle);
        if matches!(raw_handle, 0 | INVALID_HANDLE_VALUE) {
            return None;
        }

        // The mode is not important, since this call only succeeds for Windows
        // Console. Other streams usually do not require Unicode writes.
        let mut mode = 0;
        check_syscall(unsafe { GetConsoleMode(raw_handle, &mut mode) })
            .ok()
            .map(|()| Self(handle))
    }

    // Writing to the returned instance causes undefined behavior.
    #[cfg(test)]
    pub(super) const unsafe fn null() -> Self {
        // SAFETY: Null pointers can be passed to this method.
        Self(unsafe { BorrowedHandle::borrow_raw(ptr::null_mut()) })
    }

    fn write_wide(&mut self, string: &[u16]) -> io::Result<usize> {
        let length = string.len().try_into().unwrap_or(u32::MAX);
        let mut written_length = 0;
        check_syscall(unsafe {
            WriteConsoleW(
                raw_handle(self.0),
                string.as_ptr().cast(),
                length,
                &mut written_length,
                ptr::null_mut(),
            )
        })
        .map(|()| written_length as usize)
    }

    pub(super) fn write_wide_all(
        &mut self,
        mut string: &[u16],
    ) -> io::Result<()> {
        while !string.is_empty() {
            match self.write_wide(string) {
                Ok(written_length) => {
                    if written_length == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "failed to write whole buffer",
                        ));
                    }
                    string = &string[written_length..];
                }
                Err(error) => {
                    if error.kind() != io::ErrorKind::Interrupted {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}
