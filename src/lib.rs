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
//!   Provides an implementation of [`WriteBytes`] for all types.
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
// Nightly is also currently required for the SGX platform.
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_results)]

use std::io;
use std::io::BufWriter;
use std::io::LineWriter;
#[cfg(any(doc, not(feature = "specialization")))]
use std::io::Stderr;
#[cfg(any(doc, not(feature = "specialization")))]
use std::io::StderrLock;
#[cfg(any(doc, not(feature = "specialization")))]
use std::io::Stdout;
#[cfg(any(doc, not(feature = "specialization")))]
use std::io::StdoutLock;
use std::io::Write;
#[cfg(all(feature = "specialization", windows))]
use std::os::windows::io::AsRawHandle;

macro_rules! impl_write_bytes {
    ( $type:ty ) => {
        #[cfg(any(doc, not(feature = "specialization")))]
        impl $crate::WriteBytes for $type {
            #[cfg(windows)]
            fn __to_console(&self) -> Option<Console<'_>> {
                self.to_console()
            }
        }
    };
}

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

#[cfg(test)]
mod tests;

/// A bound for [`write_lossy`] that allows it to be used for some types
/// without specialization.
///
/// When the "specialization" feature is enabled, this trait is implemented for
/// all types.
pub trait WriteBytes {
    #[cfg(windows)]
    #[doc(hidden)]
    fn __to_console(&self) -> Option<Console<'_>>;
}

#[cfg(feature = "specialization")]
#[cfg_attr(print_bytes_docs_rs, doc(cfg(feature = "specialization")))]
impl<T> WriteBytes for T
where
    T: ?Sized,
{
    #[cfg(windows)]
    default fn __to_console(&self) -> Option<Console<'_>> {
        self.to_console()
    }
}

macro_rules! r#impl {
    ( $generic:ident , $type:ty ) => {
        impl<$generic> WriteBytes for $type
        where
            $generic: ?Sized + WriteBytes,
        {
            #[cfg(windows)]
            fn __to_console(&self) -> Option<Console<'_>> {
                (**self).__to_console()
            }
        }
    };
}
r#impl!(T, &mut T);
r#impl!(T, Box<T>);

macro_rules! r#impl {
    ( $generic:ident , $type:ty ) => {
        impl<$generic> WriteBytes for $type
        where
            $generic: Write + WriteBytes,
        {
            #[cfg(windows)]
            fn __to_console(&self) -> Option<Console<'_>> {
                self.get_ref().__to_console()
            }
        }
    };
}
r#impl!(T, BufWriter<T>);
r#impl!(T, LineWriter<T>);

trait ToConsole {
    #[cfg(windows)]
    fn to_console(&self) -> Option<Console<'_>>;
}

#[cfg(feature = "specialization")]
impl<T> ToConsole for T
where
    T: ?Sized,
{
    #[cfg(windows)]
    default fn to_console(&self) -> Option<Console<'_>> {
        None
    }
}

#[cfg(feature = "specialization")]
#[cfg(windows)]
impl<T> ToConsole for T
where
    T: AsRawHandle,
{
    fn to_console(&self) -> Option<Console<'_>> {
        Console::from_handle(self)
    }
}

macro_rules! r#impl {
    ( $($type:ty),+ ) => {
        $(
            impl_write_bytes!($type);

            #[cfg(not(feature = "specialization"))]
            impl ToConsole for $type {
                #[cfg(windows)]
                fn to_console(&self) -> Option<Console<'_>> {
                    Console::from_handle(self)
                }
            }
        )+
    };
}
r#impl!(Stderr, StderrLock<'_>, Stdout, StdoutLock<'_>);

impl_write_bytes!(Vec<u8>);

#[cfg(not(feature = "specialization"))]
impl ToConsole for Vec<u8> {
    #[cfg(windows)]
    fn to_console(&self) -> Option<Console<'_>> {
        None
    }
}

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
#[inline]
pub fn write_bytes<T, W>(mut writer: W, value: &T) -> io::Result<()>
where
    T: ?Sized + ToBytes,
    W: Write + WriteBytes,
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
            if let Err(error) = write_bytes($writer, value) {
                panic!("failed writing to {}: {}", $label, error);
            }
        }

        #[inline]
        $(#[$println_fn_attr])*
        pub fn $println_fn<T>(value: &T)
        where
            T: ?Sized + ToBytes,
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
