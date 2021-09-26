#![cfg(feature = "specialization")]

use std::borrow::Cow;
use std::ffi::CStr;
use std::io;
use std::io::IoSlice;
use std::io::IoSliceMut;

use print_bytes::write_bytes;

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

fn test_write(bytes: &[u8]) -> io::Result<()> {
    let mut writer = Vec::new();
    write_bytes(&mut writer, bytes)?;
    assert_eq!(bytes, writer);
    Ok(())
}

#[test]
fn test_empty_write() -> io::Result<()> {
    test_write(b"")
}

#[test]
fn test_invalid_write() -> io::Result<()> {
    test_write(INVALID_STRING)
}

#[test]
fn test_multiple_writes() -> io::Result<()> {
    let mut writer = Vec::new();

    write_bytes(&mut writer, &b"Hello, "[..])?;
    writer.extend_from_slice(b"world");
    write_bytes(&mut writer, &b"!"[..])?;

    assert_eq!(b"Hello, world!", &*writer);

    Ok(())
}

#[test]
fn test_implementations() -> io::Result<()> {
    let mut writer = Vec::new();

    write_bytes(&mut writer, &b"slice "[..])?;
    write_bytes(&mut writer, &Cow::Borrowed(&b"Cow::Borrowed "[..]))?;
    write_bytes(&mut writer, &Cow::<[_]>::Owned(b"Cow::Owned ".to_vec()))?;
    write_bytes(&mut writer, c_str(b"CStr \0"))?;
    write_bytes(&mut writer, &c_str(b"CString \0").to_owned())?;
    write_bytes(&mut writer, &IoSlice::new(b"IoSlice "))?;
    write_bytes(
        &mut writer,
        &IoSliceMut::new(&mut b"IoSliceMut ".to_owned()),
    )?;
    write_bytes(&mut writer, &b"Vec ".to_owned())?;

    assert_eq!(
        b"slice Cow::Borrowed Cow::Owned CStr CString IoSlice IoSliceMut Vec ",
        &*writer,
    );

    return Ok(());

    fn c_str(string: &[u8]) -> &CStr {
        CStr::from_bytes_with_nul(string).unwrap()
    }
}
