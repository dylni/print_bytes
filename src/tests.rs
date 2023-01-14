#![cfg(windows)]

use std::io;
use std::io::Write;

use super::console::Console;

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80";

struct Writer {
    buffer: Vec<u8>,
    is_console: bool,
}

impl Writer {
    const fn new(is_console: bool) -> Self {
        Self {
            buffer: Vec::new(),
            is_console,
        }
    }
}

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl_to_console! {
    Writer,
    // SAFETY: Since no platform strings are being written, no test should ever
    // write to this console.
    |x| x.is_console.then(|| unsafe { Console::null() }),
}

fn assert_invalid_string(writer: &Writer, lossy: bool) {
    let lossy_string = String::from_utf8_lossy(INVALID_STRING);
    let lossy_string = lossy_string.as_bytes();
    assert_ne!(INVALID_STRING, lossy_string);

    let string = &*writer.buffer;
    if lossy {
        assert_eq!(lossy_string, string);
    } else {
        assert_eq!(INVALID_STRING, string);
    }
}

#[test]
fn test_write_lossy() -> io::Result<()> {
    let mut writer = Writer::new(false);
    super::write_lossy(&mut writer, INVALID_STRING)?;
    assert_invalid_string(&writer, false);

    writer = Writer::new(true);
    super::write_lossy(&mut writer, INVALID_STRING)?;
    assert_invalid_string(&writer, true);

    Ok(())
}
