use std::ffi::OsStr;
use std::io;
use std::process::Command;
use std::process::Stdio;

use os_str_bytes::OsStrBytes;

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

#[test]
fn test_writer() -> io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_writer"))
        .arg(OsStr::from_raw_bytes(WTF8_STRING).unwrap())
        .stderr(Stdio::inherit())
        .output()?;

    if cfg!(not(windows)) {
        assert_eq!(WTF8_STRING, &*output.stdout);
    } else {
        use std::char::REPLACEMENT_CHARACTER;

        let mut replacement = [0; 4];
        let replacement = REPLACEMENT_CHARACTER.encode_utf8(&mut replacement);

        let mut lossy_string = WTF8_STRING.to_owned();
        let _ = lossy_string.splice(3..6, replacement.bytes());
        assert_ne!(WTF8_STRING, lossy_string);

        assert_eq!(lossy_string, output.stdout);
    }

    assert!(output.status.success());

    Ok(())
}