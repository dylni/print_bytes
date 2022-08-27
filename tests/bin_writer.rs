use std::char::REPLACEMENT_CHARACTER;
use std::ffi::OsStr;
use std::io;
use std::process::Command;
use std::process::Stdio;

use os_str_bytes::OsStrBytes;

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

#[test]
fn test_wtf8() -> io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_writer"))
        .arg(OsStr::assert_from_raw_bytes(WTF8_STRING))
        .stderr(Stdio::inherit())
        .output()?;

    if cfg!(windows) {
        let mut replacement = [0; 4];
        let replacement = REPLACEMENT_CHARACTER.encode_utf8(&mut replacement);

        let mut lossy_string = WTF8_STRING.to_owned();
        let _ = lossy_string.splice(3..6, replacement.bytes());
        assert_ne!(WTF8_STRING, lossy_string);

        assert_eq!(lossy_string, output.stdout);
    } else {
        assert_eq!(WTF8_STRING, &*output.stdout);
    }

    assert!(output.status.success());

    Ok(())
}
