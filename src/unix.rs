use std::os::unix::io::AsRawFd;

pub(super) fn is_console<THandle>(_: &THandle) -> bool
where
    THandle: AsRawFd,
{
    false
}
