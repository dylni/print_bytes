//! This crate allows printing broken UTF-8 bytes to an output stream as
//! losslessly as possible.
//!
//! Usually, paths are printed by calling [`Path::display`] or
//! [`Path::to_string_lossy`] beforehand. However, both of these methods are
//! always lossy; they misrepresent some valid paths in output. The same is
//! true when using [`String::from_utf8_lossy`] to print any other UTF-8–like
//! byte sequence.
//!
//! Instead, this crate only performs a lossy conversion when the output device
//! is known to require Unicode, to make output as accurate as possible. When
//! necessary, any character sequence that cannot be represented will be
//! replaced with [`REPLACEMENT_CHARACTER`]. That convention is shared with the
//! standard library, which uses the same character for its lossy conversion
//! functions.
//!
//! ### Note: Windows Compatibility
//!
//! [`OsStr`] and related structs may be printed lossily on Windows. Paths are
//! not represented using bytes on that platform, so it may be confusing to
//! display them in that manner. Plus, the encoding most often used to account
//! for the difference is [not permitted to be written to files][wtf-8
//! audience], so it would not make sense for this crate to use it.
//!
//! Windows Console can display these paths, so this crate will output them
//! losslessly when writing to that terminal.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file. Nightly features are unstable, since they rely on
//! unstable Rust features.
//!
//! ### Nightly Features
//!
//! - **specialization** -
//!   Provides [`write_bytes`].
//!
//! # Examples
//!
//! ```
//! use std::env;
//! # use std::io;
//!
//! use print_bytes::println_bytes;
//!
//! print!("exe: ");
//! println_bytes(&env::current_exe()?);
//! println!();
//!
//! println!("args:");
//! for arg in env::args_os().skip(1) {
//!     println_bytes(&arg);
//! }
//! #
//! # Ok::<_, io::Error>(())
//! ```
//!
//! [`OsStr`]: ::std::ffi::OsStr
//! [`Path::display`]: ::std::path::Path::display
//! [`Path::to_string_lossy`]: ::std::path::Path::to_string_lossy
//! [`REPLACEMENT_CHARACTER`]: ::std::char::REPLACEMENT_CHARACTER
//! [wtf-8 audience]: https://simonsapin.github.io/wtf-8/#intended-audience

#![cfg_attr(feature = "specialization", allow(incomplete_features))]
// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
#![cfg_attr(print_bytes_docs_rs, feature(doc_cfg))]
#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_results)]

use std::io;
use std::io::Stderr;
use std::io::StderrLock;
use std::io::Stdout;
use std::io::StdoutLock;
use std::io::Write;

mod bytes;
pub use bytes::ByteStr;
use bytes::ByteStrInner;
pub use bytes::ToBytes;
#[cfg(any(doc, windows))]
pub use bytes::WideStr;

#[cfg(windows)]
mod console;
#[cfg(windows)]
use console::Console;

trait WriteBytes: Write {
    #[cfg(windows)]
    fn to_console(&self) -> Option<Console<'_>>;

    #[inline]
    fn write_bytes<TValue>(&mut self, value: &TValue) -> io::Result<()>
    where
        TValue: ?Sized + ToBytes,
    {
        #[cfg_attr(not(windows), allow(unused_mut))]
        let mut lossy = false;
        #[cfg(windows)]
        if let Some(mut console) = self.to_console() {
            if let Some(string) = value.to_wide() {
                return console.write_wide_all(&string.0);
            }
            lossy = true;
        }

        let buffer;
        let string = value.to_bytes();
        let string = match &string.0 {
            ByteStrInner::Bytes(string) => {
                if lossy {
                    buffer = String::from_utf8_lossy(string);
                    buffer.as_bytes()
                } else {
                    string
                }
            }
            #[cfg(windows)]
            ByteStrInner::Str(string) => string.as_bytes(),
        };
        self.write_all(string)
    }
}

#[cfg(feature = "specialization")]
impl<T> WriteBytes for T
where
    T: ?Sized + Write,
{
    #[cfg(windows)]
    default fn to_console(&self) -> Option<Console<'_>> {
        None
    }
}

#[cfg(feature = "specialization")]
impl<'a, T> WriteBytes for &'a mut T
where
    T: ?Sized + WriteBytes,
    &'a mut T: Write,
{
    #[cfg(windows)]
    default fn to_console(&self) -> Option<Console<'_>> {
        (**self).to_console()
    }
}

macro_rules! r#impl {
    ( $($type:ty),+ ) => {
        $(
            impl WriteBytes for $type {
                #[cfg(windows)]
                fn to_console(&self) -> Option<Console<'_>> {
                    Console::from_handle(self)
                }
            }
        )+
    };
}
r#impl!(Stderr, StderrLock<'_>, Stdout, StdoutLock<'_>);

/// Writes a value to a "writer".
///
/// This function is similar to [`write!`] but does not take a format
/// parameter.
///
/// For more information, see [the module-level documentation][module].
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// [module]: self
#[cfg_attr(print_bytes_docs_rs, doc(cfg(feature = "specialization")))]
#[cfg(feature = "specialization")]
#[inline]
pub fn write_bytes<TValue, TWriter>(
    mut writer: TWriter,
    value: &TValue,
) -> io::Result<()>
where
    TValue: ?Sized + ToBytes,
    TWriter: Write,
{
    writer.write_bytes(value)
}

macro_rules! r#impl {
    (
        $writer:expr ,
        $(#[ $print_fn_attr:meta ])* $print_fn:ident ,
        $(#[ $println_fn_attr:meta ])* $println_fn:ident ,
        $label:literal $(,)?
    ) => {
        #[inline]
        $(#[$print_fn_attr])*
        pub fn $print_fn<TValue>(value: &TValue)
        where
            TValue: ?Sized + ToBytes,
        {
            if let Err(error) = $writer.write_bytes(value) {
                panic!("failed writing to {}: {}", $label, error);
            }
        }

        #[inline]
        $(#[$println_fn_attr])*
        pub fn $println_fn<TValue>(value: &TValue)
        where
            TValue: ?Sized + ToBytes,
        {
            let _ = $writer.lock();
            $print_fn(value);
            $print_fn(&b"\n"[..]);
        }
    };
}
r#impl!(
    io::stdout(),
    /// Prints a value to the standard output stream.
    ///
    /// This function is similar to [`print!`] but does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: self
    print_bytes,
    /// Prints a value to the standard output stream, followed by a newline.
    ///
    /// This function is similar to [`println!`] but does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: self
    println_bytes,
    "stdout",
);
r#impl!(
    io::stderr(),
    /// Prints a value to the standard error stream.
    ///
    /// This function is similar to [`eprint!`] but does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: self
    eprint_bytes,
    /// Prints a value to the standard error stream, followed by a newline.
    ///
    /// This function is similar to [`eprintln!`] but does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: self
    eprintln_bytes,
    "stderr",
);

#[cfg(all(test, windows))]
mod tests {
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
            // SAFETY: Since no platform strings are being written, no test
            // should ever write to this console.
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

    fn test<TWriteFn>(mut write_fn: TWriteFn) -> io::Result<()>
    where
        TWriteFn: FnMut(&mut Writer, &[u8]) -> io::Result<()>,
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
}
