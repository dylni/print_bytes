use std::borrow::Cow;
use std::ffi::CStr;
#[cfg(feature = "os_str_bytes")]
use std::ffi::OsStr;
use std::io;
#[cfg(feature = "os_str_bytes")]
use std::path::Path;

use print_bytes::write_lossy;

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

fn test_write(bytes: &[u8]) -> io::Result<()> {
    let mut writer = Vec::new();
    write_lossy(&mut writer, bytes)?;
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

    write_lossy(&mut writer, &b"Hello, "[..])?;
    writer.extend_from_slice(b"world");
    write_lossy(&mut writer, &b"!"[..])?;

    assert_eq!(b"Hello, world!", &*writer);

    Ok(())
}

#[test]
fn test_implementations() -> io::Result<()> {
    const C_STRING: &CStr = c"foobar";
    const STRING: &str = if let Ok(string) = &C_STRING.to_str() {
        string
    } else {
        unreachable!();
    };
    const STRING_BYTES: &[u8] = STRING.as_bytes();

    macro_rules! test_one {
        ( $value:expr ) => {{
            let mut writer = Vec::new();
            write_lossy(&mut writer, $value)?;
            assert_eq!(STRING_BYTES, &*writer);
        }};
    }

    macro_rules! test {
        ( $value:expr ) => {{
            let value = $value;
            test_one!(value);
            test_one!(&value.to_owned());
        }};
    }

    test!(C_STRING);
    test!(STRING_BYTES);
    #[cfg(feature = "os_str_bytes")]
    {
        test!(OsStr::new(STRING));
        test!(Path::new(STRING));
    }

    test_one!(&Cow::Borrowed(STRING_BYTES));
    test_one!(&Cow::<[_]>::Owned(STRING_BYTES.to_owned()));

    Ok(())
}
