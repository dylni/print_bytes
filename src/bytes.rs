use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::IoSlice;
use std::io::IoSliceMut;
use std::ops::Deref;

/// A value that can be printed by any of the functions in this crate.
///
/// For more information, see [`ToBytes`].
///
/// [`ToBytes`]: trait.ToBytes.html
#[derive(Debug)]
pub struct Bytes<'a>(pub(crate) Cow<'a, [u8]>);

impl<'a, T> From<Cow<'a, T>> for Bytes<'a>
where
    T: ?Sized + ToBytes<'a> + ToOwned,
    T::Owned: Into<Bytes<'a>>,
{
    #[inline]
    fn from(value: Cow<'a, T>) -> Self {
        match value {
            Cow::Borrowed(value) => value.to_bytes(),
            Cow::Owned(value) => value.into(),
        }
    }
}

impl From<Vec<u8>> for Bytes<'static> {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Bytes(Cow::Owned(value))
    }
}

macro_rules! r#impl {
    ( $type:ty , $convert_method:ident ) => {
        impl From<$type> for Bytes<'static> {
            #[inline]
            fn from(value: $type) -> Self {
                Into::into(value.$convert_method())
            }
        }
    };
}

r#impl!(CString, into_bytes);

#[cfg(feature = "os_str")]
mod os_string {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use os_str_bytes::OsStringBytes;

    use super::Bytes;

    r#impl!(OsString, into_vec);

    r#impl!(PathBuf, into_os_string);
}

/// Represents a type similarly to [`Display`].
///
/// Implement this trait to allow printing a type that cannot guarantee UTF-8
/// output. It is used to bound values accepted by functions in this crate.
///
/// Since [`Bytes`] has no public constructor, implementations should call
/// [`to_bytes`] on another type to create an instance. If storing owned bytes
/// is necessary, [`Bytes::from`] can be used instead to avoid lifetime errors.
///
/// Consider also implementing [`From`] for [`Bytes`] when the type is a smart
/// pointer.
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
/// impl<'a> ToBytes<'a> for ByteSlice<'a> {
///     fn to_bytes(&'a self) -> Bytes<'a> {
///         self.0.to_bytes()
///     }
/// }
///
/// println_bytes(&ByteSlice(b"Hello, world!"));
/// ```
///
/// [`Bytes`]: struct.Bytes.html
/// [`Bytes::from`]: struct.Bytes.html#impl-From%3CVec%3Cu8%3E%3E
/// [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [`to_bytes`]: #tymethod.to_bytes
/// [`ToString`]: https://doc.rust-lang.org/std/string/trait.ToString.html
pub trait ToBytes<'a> {
    /// Creates a byte sequence that will be used to represent the instance.
    #[must_use]
    fn to_bytes(&'a self) -> Bytes<'a>;
}

impl<'a> ToBytes<'a> for [u8] {
    #[inline]
    fn to_bytes(&'a self) -> Bytes<'a> {
        Bytes(Cow::Borrowed(self))
    }
}

#[cfg(any(all(doc, not(doctest)), feature = "const_generics"))]
impl<'a, const N: usize> ToBytes<'a> for [u8; N] {
    #[inline]
    fn to_bytes(&'a self) -> Bytes<'a> {
        self.as_ref().to_bytes()
    }
}

impl<'a, T> ToBytes<'a> for Cow<'a, T>
where
    T: ?Sized + ToBytes<'a> + ToOwned,
    T::Owned: ToBytes<'a>,
{
    #[inline]
    fn to_bytes(&'a self) -> Bytes<'a> {
        match self {
            Cow::Borrowed(value) => value.to_bytes(),
            Cow::Owned(value) => value.to_bytes(),
        }
    }
}

macro_rules! r#impl {
    ( $type:ty , $convert_method:ident ) => {
        impl<'a> ToBytes<'a> for $type {
            #[inline]
            fn to_bytes(&'a self) -> Bytes<'a> {
                ToBytes::to_bytes(self.$convert_method())
            }
        }
    };
}

r#impl!(CStr, to_bytes);

r#impl!(CString, as_c_str);

r#impl!(IoSlice<'a>, deref);

r#impl!(IoSliceMut<'a>, deref);

r#impl!(Vec::<u8>, as_slice);

#[cfg(feature = "os_str")]
mod os_str {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use super::Bytes;
    use super::ToBytes;

    impl<'a> ToBytes<'a> for OsStr {
        #[inline]
        fn to_bytes(&'a self) -> Bytes<'a> {
            use os_str_bytes::OsStrBytes;

            Bytes(OsStrBytes::to_bytes(self))
        }
    }

    r#impl!(OsString, as_os_str);

    r#impl!(Path, as_os_str);

    r#impl!(PathBuf, as_path);
}
