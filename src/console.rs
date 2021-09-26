use std::convert::TryInto;
use std::io;
use std::marker::PhantomData;
use std::os::windows::io::AsRawHandle;
use std::ptr;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::TRUE;
use winapi::um::consoleapi::GetConsoleMode;
use winapi::um::consoleapi::WriteConsoleW;
use winapi::um::winnt::HANDLE;

pub(super) struct Console<'a> {
    handle: HANDLE,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Console<'a> {
    pub(super) fn from_handle<THandle>(handle: &'a THandle) -> Option<Self>
    where
        THandle: AsRawHandle,
    {
        let handle = handle.as_raw_handle() as HANDLE;
        // The mode is not important, since this call only succeeds for Windows
        // Console. Other streams usually do not require Unicode writes.
        let mut mode = 0;
        if unsafe { GetConsoleMode(handle, &mut mode) } == TRUE {
            Some(Self {
                handle,
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    // Writing to the returned instance causes undefined behavior.
    #[cfg(test)]
    pub(super) const unsafe fn null() -> Self {
        Self {
            handle: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    fn write_wide(&mut self, string: &[u16]) -> io::Result<usize> {
        let length = string
            .len()
            .try_into()
            .unwrap_or_else(|_| DWORD::max_value());
        let mut written_length = 0;
        let result = unsafe {
            WriteConsoleW(
                self.handle,
                string.as_ptr() as *const c_void,
                length,
                &mut written_length,
                ptr::null_mut(),
            )
        };
        if result == TRUE {
            Ok(written_length as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(super) fn write_wide_all(
        &mut self,
        mut string: &[u16],
    ) -> io::Result<()> {
        while !string.is_empty() {
            let written_length = self.write_wide(string)?;
            string = &string[written_length..];
        }
        Ok(())
    }
}
