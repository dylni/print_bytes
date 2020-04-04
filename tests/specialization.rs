#![cfg(feature = "specialization")]

use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::IoSlice;
use std::io::IoSliceMut;
use std::io::Result as IoResult;

use print_bytes::write_bytes;

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

fn test_write(bytes: &[u8]) -> IoResult<()> {
    let mut writer = Vec::new();
    write_bytes(&mut writer, bytes)?;
    assert_eq!(bytes, writer.as_slice());
    Ok(())
}

#[test]
fn test_empty_write() -> IoResult<()> {
    test_write(&[])
}

#[test]
fn test_invalid_write() -> IoResult<()> {
    test_write(INVALID_STRING)
}

#[test]
fn test_multiple_writes() -> IoResult<()> {
    let mut writer = Vec::new();

    write_bytes(&mut writer, b"Hello, ".as_ref())?;
    writer.extend_from_slice(b"world");
    write_bytes(&mut writer, b"!".as_ref())?;

    assert_eq!(b"Hello, world!", writer.as_slice());

    Ok(())
}

#[test]
fn test_implementations() -> IoResult<()> {
    let mut writer = Vec::new();

    write_bytes(&mut writer, b"slice ".as_ref())?;
    write_bytes(&mut writer, &Cow::Borrowed(b"Cow::Borrowed ".as_ref()))?;
    write_bytes(&mut writer, &Cow::<[u8]>::Owned(b"Cow::Owned ".to_vec()))?;
    write_bytes(&mut writer, CStr::from_bytes_with_nul(b"CStr \0").unwrap())?;
    write_bytes(&mut writer, &CString::new(b"CString ".as_ref())?)?;
    write_bytes(&mut writer, &IoSlice::new(b"IoSlice "))?;
    write_bytes(&mut writer, &IoSliceMut::new(&mut b"IoSliceMut ".to_vec()))?;
    write_bytes(&mut writer, &b"Vec ".to_vec())?;

    assert_eq!(
        &b"slice Cow::Borrowed Cow::Owned CStr CString IoSlice IoSliceMut Vec "[..],
        writer.as_slice(),
    );

    Ok(())
}
