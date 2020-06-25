use std::io;
use std::os::unix::io::AsRawFd;

use super::WriteBytes;

pub(crate) fn is_console<THandle>(_: &THandle) -> bool
where
    THandle: AsRawFd,
{
    false
}

pub(crate) fn write<TWriter>(
    writer: &mut TWriter,
    bytes: &[u8],
) -> io::Result<()>
where
    TWriter: ?Sized + WriteBytes,
{
    writer.write_all(bytes)
}
