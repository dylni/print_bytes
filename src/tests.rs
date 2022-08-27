#![cfg(windows)]

use std::io;
use std::io::Write;

use super::Console;
use super::WriteBytes;

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
    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.write(bytes)
    }
}

impl WriteBytes for Writer {
    fn to_console(&self) -> Option<Console<'_>> {
        // SAFETY: Since no platform strings are being written, no test should
        // ever write to this console.
        self.is_console.then(|| unsafe { Console::null() })
    }
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

fn test<F>(mut write_fn: F) -> io::Result<()>
where
    F: FnMut(&mut Writer, &[u8]) -> io::Result<()>,
{
    let mut writer = Writer::new(false);
    write_fn(&mut writer, INVALID_STRING)?;
    assert_invalid_string(&writer, false);

    writer = Writer::new(true);
    write_fn(&mut writer, INVALID_STRING)?;
    assert_invalid_string(&writer, true);

    Ok(())
}

#[test]
fn test_write() -> io::Result<()> {
    test(WriteBytes::write_bytes)
}

#[cfg(feature = "specialization")]
#[test]
fn test_write_bytes() -> io::Result<()> {
    test(|writer, bytes| super::write_bytes(writer, bytes))
}
