use std::os::windows::io::AsRawHandle;

use winapi::um::consoleapi::GetConsoleMode;
use winapi::um::winnt::HANDLE;

pub(super) fn is_console<THandle>(handle: &THandle) -> bool
where
    THandle: AsRawHandle,
{
    let handle = handle.as_raw_handle() as HANDLE;
    // The mode is not important, but this call only succeeds for Windows
    // Console. Other streams usually do not require unicode writes.
    let mut mode = 0;
    unsafe { GetConsoleMode(handle, &mut mode) != 0 }
}
