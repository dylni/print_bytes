#![cfg(feature = "os_str_bytes")]

use std::char::REPLACEMENT_CHARACTER;
use std::io;
use std::process::Command;
use std::process::Stdio;

#[test]
fn test_wtf8() -> io::Result<()> {
    #[cfg(windows)]
    let buffer;
    let string = {
        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            OsStr::from_bytes(b"\x66\x6F\x80\x6F")
        }
        #[cfg(windows)]
        {
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;

            buffer = OsString::from_wide(&[0x66, 0x6F, 0xD800, 0x6F]);
            &buffer
        }
    };
    assert_eq!(None, string.to_str());

    let output = Command::new(env!("CARGO_BIN_EXE_writer"))
        .arg(string)
        .stderr(Stdio::inherit())
        .output()?;

    if cfg!(windows) {
        assert_eq!(
            format!("\x66\x6F{}\x6F", REPLACEMENT_CHARACTER).as_bytes(),
            output.stdout,
        );
    } else {
        assert_eq!(&b"\x66\x6F\x80\x6F"[..], output.stdout);
    }

    assert!(output.status.success());

    Ok(())
}
