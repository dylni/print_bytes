use std::convert::TryInto;
use std::io;
use std::marker::PhantomData;
use std::os::windows::io::AsRawHandle;
use std::ptr;

use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Console::GetConsoleMode;
use windows_sys::Win32::System::Console::WriteConsoleW;

const TRUE: BOOL = 1;

pub struct Console<'a> {
    handle: HANDLE,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Console<'a> {
    pub(super) fn from_handle<T>(handle: &'a T) -> Option<Self>
    where
        T: AsRawHandle + ?Sized,
    {
        let handle = handle.as_raw_handle();
        if handle.is_null() {
            return None;
        }
        let handle = handle as _;

        // The mode is not important, since this call only succeeds for Windows
        // Console. Other streams usually do not require Unicode writes.
        let mut mode = 0;
        (unsafe { GetConsoleMode(handle, &mut mode) } == TRUE).then(|| Self {
            handle,
            _marker: PhantomData,
        })
    }

    // Writing to the returned instance causes undefined behavior.
    #[cfg(test)]
    pub(super) const unsafe fn null() -> Self {
        Self {
            handle: 0,
            _marker: PhantomData,
        }
    }

    fn write_wide(&mut self, string: &[u16]) -> io::Result<usize> {
        let length = string.len().try_into().unwrap_or(u32::MAX);
        let mut written_length = 0;
        let result = unsafe {
            WriteConsoleW(
                self.handle,
                string.as_ptr().cast(),
                length,
                &mut written_length,
                ptr::null_mut(),
            )
        };
        (result == TRUE)
            .then(|| written_length as usize)
            .ok_or_else(io::Error::last_os_error)
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
