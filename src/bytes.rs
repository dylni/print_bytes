use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::IoSlice;
use std::io::IoSliceMut;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub(super) enum BytesInner<'a> {
    Bytes(&'a [u8]),
    OsStr(&'a OsStr),
}

/// A value that can be printed by any of the functions in this crate.
///
/// For more information, see [`ToBytes`].
#[derive(Debug)]
pub struct Bytes<'a>(pub(super) BytesInner<'a>);

/// Represents a type similarly to [`Display`].
///
/// Implement this trait to allow printing a type that cannot guarantee UTF-8
/// output. It is used to bound values accepted by functions in this crate.
///
/// Since [`Bytes`] has no public constructor, implementations should call
/// [`to_bytes`] on another type to create an instance.
///
/// ### Note: Future Compatibility
///
/// This trait should not be implemented for types that also implement
/// [`Display`] or [`ToString`]. A blanket implementation may eventually be
/// provided to cover those traits automatically, but that would currently be
/// considered a coherence violation.
///
/// # Examples
///
/// ```
/// use print_bytes::println_bytes;
/// use print_bytes::Bytes;
/// use print_bytes::ToBytes;
///
/// struct ByteSlice<'a>(&'a [u8]);
///
/// impl ToBytes for ByteSlice<'_> {
///     fn to_bytes(&self) -> Bytes<'_> {
///         self.0.to_bytes()
///     }
/// }
///
/// println_bytes(&ByteSlice(b"Hello, world!"));
/// ```
///
/// [`Display`]: ::std::fmt::Display
/// [`to_bytes`]: Self::to_bytes
/// [`ToString`]: ::std::string::ToString
pub trait ToBytes {
    /// Creates a byte sequence that will be used to represent the instance.
    #[must_use]
    fn to_bytes(&self) -> Bytes<'_>;
}

impl ToBytes for [u8] {
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        Bytes(BytesInner::Bytes(self))
    }
}

#[cfg_attr(print_bytes_docs_rs, doc(cfg(feature = "min_const_generics")))]
#[cfg(feature = "min_const_generics")]
impl<const N: usize> ToBytes for [u8; N] {
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        self[..].to_bytes()
    }
}

impl<T> ToBytes for Cow<'_, T>
where
    T: ?Sized + ToBytes + ToOwned,
    T::Owned: ToBytes,
{
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        (**self).to_bytes()
    }
}

impl ToBytes for OsStr {
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        Bytes(BytesInner::OsStr(self))
    }
}

macro_rules! r#impl {
    ( $type:ty , $convert_method:ident ) => {
        impl ToBytes for $type {
            #[inline]
            fn to_bytes(&self) -> Bytes<'_> {
                ToBytes::to_bytes(self.$convert_method())
            }
        }
    };
}
r#impl!(CStr, to_bytes);
r#impl!(CString, as_c_str);
r#impl!(IoSlice<'_>, deref);
r#impl!(IoSliceMut<'_>, deref);
r#impl!(OsString, as_os_str);
r#impl!(Path, as_os_str);
r#impl!(PathBuf, as_path);
r#impl!(Vec::<u8>, as_slice);
