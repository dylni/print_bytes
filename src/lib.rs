//! This crate allows printing broken UTF-8 bytes to an output stream as
//! losslessly as possible.
//!
//! Usually, paths are printed by calling [`Path::display`] or
//! [`Path::to_string_lossy`] beforehand. However, both of these methods are
//! always lossy; they misrepresent some valid paths in output. The same is
//! true when using [`String::from_utf8_lossy`] to print any other
//! UTF-8&ndash;like byte sequence.
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
//!   Provides an implementation of [`WriteLossy`] for all types.
//!
//! # Examples
//!
//! ```
//! use std::env;
//! # use std::io;
//!
//! use print_bytes::println_lossy;
//!
//! print!("exe: ");
//! println_lossy(&env::current_exe()?);
//! println!();
//!
//! println!("args:");
//! for arg in env::args_os().skip(1) {
//!     println_lossy(&arg);
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
// Nightly is also currently required for the SGX platform.
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(unused_results)]

use std::io;
use std::io::Write;

mod bytes;
pub use bytes::ByteStr;
use bytes::ByteStrInner;
pub use bytes::ToBytes;
#[cfg(any(doc, windows))]
pub use bytes::WideStr;

#[cfg(windows)]
mod console;

#[cfg_attr(test, macro_use)]
mod writer;
pub use writer::WriteLossy;

#[cfg(test)]
mod tests;

/// Writes a value to a "writer".
///
/// This function is similar to [`write!`] but does not take a format
/// parameter.
///
/// For more information, see [the module-level documentation][module].
///
/// # Errors
///
/// Returns an error if writing to the writer fails.
///
/// # Examples
///
/// ```
/// use std::env;
/// use std::ffi::OsStr;
///
/// use print_bytes::write_lossy;
///
/// let string = "foobar";
/// let os_string = OsStr::new(string);
///
/// let mut lossy_string = Vec::new();
/// write_lossy(&mut lossy_string, os_string)
///     .expect("failed writing to vector");
/// assert_eq!(string.as_bytes(), lossy_string);
/// ```
///
/// [module]: self
#[inline]
pub fn write_lossy<T, W>(mut writer: W, value: &T) -> io::Result<()>
where
    T: ?Sized + ToBytes,
    W: Write + WriteLossy,
{
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut lossy = false;
    #[cfg(windows)]
    if let Some(mut console) = writer.__to_console() {
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
    writer.write_all(string)
}

macro_rules! expect_print {
    ( $label:literal , $result:expr ) => {
        $result
            .unwrap_or_else(|x| panic!("failed writing to {}: {}", $label, x))
    };
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
        pub fn $print_fn<T>(value: &T)
        where
            T: ?Sized + ToBytes,
        {
            expect_print!($label, write_lossy($writer, value));
        }

        #[inline]
        $(#[$println_fn_attr])*
        pub fn $println_fn<T>(value: &T)
        where
            T: ?Sized + ToBytes,
        {
            let mut writer = $writer.lock();
            expect_print!($label, write_lossy(&mut writer, value));
            expect_print!($label, writer.write_all(b"\n"));
        }
    };
}
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
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use print_bytes::eprint_lossy;
    ///
    /// eprint_lossy(&env::current_exe()?);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [module]: self
    eprint_lossy,
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
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use print_bytes::eprintln_lossy;
    ///
    /// eprintln_lossy(&env::current_exe()?);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [module]: self
    eprintln_lossy,
    "stderr",
);
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
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use print_bytes::print_lossy;
    ///
    /// print_lossy(&env::current_exe()?);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [module]: self
    print_lossy,
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
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use print_bytes::println_lossy;
    ///
    /// println_lossy(&env::current_exe()?);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [module]: self
    println_lossy,
    "stdout",
);
