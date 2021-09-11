use std::ffi::OsStr;
use std::io;
use std::marker::PhantomData;

#[cfg(any(target_os = "hermit", unix))]
use std::os::unix as os;
#[cfg(target_os = "wasi")]
use std::os::wasi as os;

use os::ffi::OsStrExt;
use os::io::AsRawFd;

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
