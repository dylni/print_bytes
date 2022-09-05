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
// Nightly is also currently required for the SGX platform.
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_results)]

#[cfg(feature = "specialization")]
use std::io;
#[cfg(feature = "specialization")]
use std::io::Write;

pub use print_bytes2::eprint_lossy as eprint_bytes;
pub use print_bytes2::eprintln_lossy as eprintln_bytes;
pub use print_bytes2::print_lossy as print_bytes;
pub use print_bytes2::println_lossy as println_bytes;
pub use print_bytes2::ByteStr;
pub use print_bytes2::ToBytes;
#[cfg(windows)]
pub use print_bytes2::WideStr;

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
pub fn write_bytes<T, W>(writer: W, value: &T) -> io::Result<()>
where
    T: ?Sized + ToBytes,
    W: Write,
{
    print_bytes2::write_lossy(writer, value)
}
