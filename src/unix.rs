use std::ffi::OsStr;
use std::io;
use std::marker::PhantomData;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

use super::WriteBytes;

pub(super) struct Console<'a>(PhantomData<&'a ()>);

impl<'a> Console<'a> {
    pub(super) fn from_handle<THandle>(_: &'a THandle) -> Option<Self>
    where
        THandle: AsRawFd,
    {
        None
    }

    #[cfg(test)]
    pub(super) const unsafe fn null() -> Self {
        Self(PhantomData)
    }
}

pub(super) fn write_os<TWriter>(
    writer: &mut TWriter,
    os_string: &OsStr,
) -> io::Result<()>
where
    TWriter: ?Sized + WriteBytes,
{
    writer.write_all(os_string.as_bytes())
}
