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
//! is known to require unicode, to make output as accurate as possible. When
//! necessary, any character sequence that cannot be represented will be
//! replaced with [`REPLACEMENT_CHARACTER`]. That convention is shared with the
//! standard library, which uses the same character for its lossy conversion
//! functions.
//!
//! ### Note: Windows Compatibility
//!
//! [`OsStr`] and related structs will be printed lossily on Windows. Paths are
//! not represented using bytes on that platform, so it may be confusing to
//! display them in that manner. Plus, the encoding most often used to account
//! for the difference is [not permitted to be written to files][wtf-8
//! audience], so it would not make sense for this crate to use it.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file. Nightly features are unstable, since they rely on
//! unstable Rust features.
//!
//! ### Nightly Features
//!
//! - **const_generics** -
//!   Provides an implementation of [`ToBytes`] for [`[u8; N]`][array]. As a
//!   result, it can be output using functions in this crate.
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
//! # fn main() -> io::Result<()> {
//! print!("exe: ");
//! println_bytes(&env::current_exe()?);
//! println!();
//!
//! println!("args:");
//! for arg in env::args_os().skip(1) {
//!     println_bytes(&arg);
//! }
//! #     Ok(())
//! # }
//! ```
//!
//! [array]: https://doc.rust-lang.org/std/primitive.array.html
//! [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
//! [`Path::display`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.display
//! [`Path::to_string_lossy`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.to_string_lossy
//! [`REPLACEMENT_CHARACTER`]: https://doc.rust-lang.org/std/char/constant.REPLACEMENT_CHARACTER.html
//! [`String::from_utf8_lossy`]: https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy
//! [`ToBytes`]: trait.ToBytes.html
//! [`write_bytes`]: fn.write_bytes.html
//! [wtf-8 audience]: https://simonsapin.github.io/wtf-8/#intended-audience

#![cfg_attr(
    any(feature = "const_generics", feature = "specialization"),
    allow(incomplete_features)
)]
#![doc(html_root_url = "https://docs.rs/print_bytes/*")]
#![cfg_attr(any(feature = "const_generics"), feature(const_generics))]
// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
#![cfg_attr(print_bytes_docs_rs, feature(doc_cfg))]
#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(unused_results)]

use std::io;
use std::io::Stderr;
use std::io::StderrLock;
use std::io::Stdout;
use std::io::StdoutLock;
use std::io::Write;

mod bytes;
pub use bytes::Bytes;
pub use bytes::ToBytes;

#[cfg(unix)]
#[path = "unix.rs"]
mod imp;
#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

trait WriteBytes: Write {
    fn is_console(&self) -> bool;

    #[inline]
    fn write_bytes<'a, TValue>(&mut self, value: &'a TValue) -> io::Result<()>
    where
        TValue: ?Sized + ToBytes<'a>,
    {
        imp::write(self, &value.to_bytes().0)
    }
}

#[cfg(feature = "specialization")]
impl<T> WriteBytes for T
where
    T: ?Sized + Write,
{
    default fn is_console(&self) -> bool {
        false
    }
}

macro_rules! r#impl {
    ( $($type:ty),* $(,)? ) => {
        $(
            impl WriteBytes for $type {
                fn is_console(&self) -> bool {
                    imp::is_console(self)
                }
            }
        )*
    };
}
r#impl!(Stderr, StderrLock<'_>, Stdout, StdoutLock<'_>);

/// Writes a value to a "writer".
///
/// This function is similar to [`write!`], but it does not take a format
/// parameter.
///
/// For more information, see [the module-level documentation][module].
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// [module]: index.html
/// [`write!`]: https://doc.rust-lang.org/std/macro.write.html
#[cfg_attr(print_bytes_docs_rs, doc(cfg(feature = "specialization")))]
#[cfg(feature = "specialization")]
#[inline]
pub fn write_bytes<'a, TValue, TWriter>(
    writer: &mut TWriter,
    value: &'a TValue,
) -> io::Result<()>
where
    TValue: ?Sized + ToBytes<'a>,
    TWriter: ?Sized + Write,
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
        $(#[$print_fn_attr])*
        #[inline]
        pub fn $print_fn<'a, TValue>(value: &'a TValue)
        where
            TValue: ?Sized + ToBytes<'a>,
        {
            if let Err(error) = $writer.write_bytes(value) {
                panic!("failed writing to {}: {}", $label, error);
            }
        }

        $(#[$println_fn_attr])*
        #[inline]
        pub fn $println_fn<'a, TValue>(value: &'a TValue)
        where
            TValue: ?Sized + ToBytes<'a>,
        {
            let _ = $writer.lock();
            $print_fn(value);
            $print_fn(b"\n".as_ref());
        }
    };
}
r#impl!(
    io::stdout(),
    /// Prints a value to the standard output stream.
    ///
    /// This function is similar to [`print!`], but it does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: index.html
    /// [`print!`]: https://doc.rust-lang.org/std/macro.print.html
    print_bytes,
    /// Prints a value to the standard output stream, followed by a newline.
    ///
    /// This function is similar to [`println!`], but it does not take a format
    /// parameter. A line feed (`\n`) is still used for the newline, with no
    /// additional carriage return (`\r`) printed.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [module]: index.html
    /// [`println!`]: https://doc.rust-lang.org/std/macro.println.html
    println_bytes,
    "stdout",
);
r#impl!(
    io::stderr(),
    /// Prints a value to the standard error stream.
    ///
    /// This function is similar to [`eprint!`], but it does not take a format
    /// parameter.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [`eprint!`]: https://doc.rust-lang.org/std/macro.eprint.html
    /// [module]: index.html
    eprint_bytes,
    /// Prints a value to the standard error stream, followed by a newline.
    ///
    /// This function is similar to [`eprintln!`], but it does not take a
    /// format parameter. A line feed (`\n`) is still used for the newline,
    /// with no additional carriage return (`\r`) printed.
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// # Panics
    ///
    /// Panics if writing to the stream fails.
    ///
    /// [`eprintln!`]: https://doc.rust-lang.org/std/macro.eprintln.html
    /// [module]: index.html
    eprintln_bytes,
    "stderr",
);

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Write;

    use super::imp;
    use super::WriteBytes;

    const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

    struct Writer {
        buffer: Vec<u8>,
        is_console: bool,
    }

    impl Writer {
        fn new(is_console: bool) -> Self {
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
        fn is_console(&self) -> bool {
            self.is_console
        }
    }

    fn assert_invalid_string(writer: &Writer, lossy: bool) {
        let bytes = writer.buffer.as_slice();
        assert_ne!(lossy, INVALID_STRING == bytes);
        if lossy {
            assert_eq!(
                String::from_utf8_lossy(INVALID_STRING).as_bytes(),
                bytes,
            );
        }
    }

    fn test<TWriteFn>(mut write_fn: TWriteFn) -> io::Result<()>
    where
        TWriteFn: FnMut(&mut Writer, &[u8]) -> io::Result<()>,
    {
        let mut writer = Writer::new(true);
        write_fn(&mut writer, INVALID_STRING)?;
        assert_invalid_string(&writer, cfg!(windows));

        writer = Writer::new(false);
        write_fn(&mut writer, INVALID_STRING)?;
        assert_invalid_string(&writer, false);

        Ok(())
    }

    #[test]
    fn test_write() -> io::Result<()> {
        test(imp::write)
    }

    #[cfg(feature = "specialization")]
    #[test]
    fn test_write_bytes() -> io::Result<()> {
        test(|writer, bytes| super::write_bytes(writer, bytes))
    }
}
