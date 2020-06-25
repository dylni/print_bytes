use std::ffi::OsStr;
use std::io;
use std::process::Command;
use std::process::Stdio;

use os_str_bytes::OsStrBytes;

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

#[test]
fn test_process_pipe() -> io::Result<()> {
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

    let output = output.stdout.as_slice();
    #[cfg(unix)]
    assert_eq!(WTF8_STRING, output);
    #[cfg(windows)]
    {
        use std::char::REPLACEMENT_CHARACTER;

        assert_ne!(WTF8_STRING, output);

        let mut replacement = [0; 4];
        let replacement = REPLACEMENT_CHARACTER.encode_utf8(&mut replacement);

        let mut lossy_string = WTF8_STRING.to_vec();
        let _ = lossy_string.splice(3..6, replacement.bytes());
        assert_eq!(lossy_string, output);
    }

    Ok(())
}

#[cfg(feature = "specialization")]
#[test]
fn test_implementations() -> io::Result<()> {
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
