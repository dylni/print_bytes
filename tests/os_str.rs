#![cfg(feature = "os_str")]

use std::ffi::OsStr;
use std::io::Result as IoResult;
use std::process::Command;
use std::process::Stdio;

use os_str_bytes::OsStrBytes;

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

#[test]
fn test_process_pipe() -> IoResult<()> {
    let output = Command::new(format!(
        "target/{}/writer",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    ))
    .arg(OsStr::from_bytes(WTF8_STRING).unwrap())
    .stdout(Stdio::piped())
    .spawn()?
    .wait_with_output()?;

    assert_eq!(WTF8_STRING, output.stdout.as_slice());

    Ok(())
}

#[cfg(feature = "specialization")]
#[test]
fn test_implementations() -> IoResult<()> {
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use print_bytes::write_bytes;

    let mut writer = Vec::new();

    write_bytes(&mut writer, OsStr::new("OsStr "))?;
    write_bytes(&mut writer, &OsString::from("OsString "))?;
    write_bytes(&mut writer, Path::new("Path "))?;
    write_bytes(&mut writer, &PathBuf::from("PathBuf "))?;

    assert_eq!(b"OsStr OsString Path PathBuf ", writer.as_slice());

    Ok(())
}
