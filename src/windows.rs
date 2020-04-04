use std::io::Result as IoResult;
use std::os::windows::io::AsRawHandle;

use winapi::um::consoleapi::GetConsoleMode;
use winapi::um::winnt::HANDLE;

use super::WriteBytes;

pub(crate) fn is_console<THandle>(handle: &THandle) -> bool
where
    THandle: AsRawHandle,
{
    let handle = handle.as_raw_handle() as HANDLE;
    // The mode is not important, but this call only succeeds for Windows
    // Console. Other streams usually do not require unicode writes.
    let mut mode = 0;
    unsafe { GetConsoleMode(handle, &mut mode) != 0 }
}

pub(crate) fn write<TWriter>(
    writer: &mut TWriter,
    bytes: &[u8],
) -> IoResult<()>
where
    TWriter: ?Sized + WriteBytes,
{
    let buffer;
    let bytes = if (*writer).is_console() {
        buffer = String::from_utf8_lossy(bytes);
        buffer.as_bytes()
    } else {
        bytes
    };
    writer.write_all(bytes)
}
