use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::ops::Deref;

#[derive(Debug)]
pub(super) enum ByteStrInner<'a> {
    Bytes(&'a [u8]),
    #[cfg(windows)]
    Str(Cow<'a, str>),
}

/// A value returned by [`ToBytes::to_bytes`].
///
/// This struct is usually initialized by calling the above method for
/// [`[u8]`][slice].
#[derive(Debug)]
pub struct ByteStr<'a>(pub(super) ByteStrInner<'a>);

#[cfg(any(doc, windows))]
#[cfg_attr(print_bytes_docs_rs, doc(cfg(windows)))]
impl<'a> ByteStr<'a> {
    /// Wraps a byte string lossily.
    ///
    /// This method can be used to implement [`ToBytes::to_bytes`] when
    /// [`ToBytes::to_wide`] is the better way to represent the string.
    #[inline]
    #[must_use]
    pub fn from_utf8_lossy(string: &'a [u8]) -> Self {
        Self(ByteStrInner::Str(String::from_utf8_lossy(string)))
    }
}

/// A value returned by [`ToBytes::to_wide`].
#[cfg(any(doc, windows))]
#[cfg_attr(print_bytes_docs_rs, doc(cfg(windows)))]
#[derive(Debug)]
pub struct WideStr(pub(super) Vec<u16>);

#[cfg(any(doc, windows))]
impl WideStr {
    /// Wraps a wide character string.
    ///
    /// This method can be used to implement [`ToBytes::to_wide`].
    #[inline]
    #[must_use]
    pub fn new(string: Vec<u16>) -> Self {
        Self(string)
    }
}

/// Represents a type similarly to [`Display`].
///
/// Implement this trait to allow printing a type that cannot guarantee UTF-8
/// output. It is used to bound values accepted by functions in this crate.
///
/// # Examples
///
/// ```
/// use print_bytes::println_lossy;
/// use print_bytes::ByteStr;
/// use print_bytes::ToBytes;
/// #[cfg(windows)]
/// use print_bytes::WideStr;
///
/// struct ByteSlice<'a>(&'a [u8]);
///
/// impl ToBytes for ByteSlice<'_> {
///     fn to_bytes(&self) -> ByteStr<'_> {
///         self.0.to_bytes()
///     }
///
///     #[cfg(windows)]
///     fn to_wide(&self) -> Option<WideStr> {
///         self.0.to_wide()
///     }
/// }
///
/// println_lossy(&ByteSlice(b"Hello, world!"));
/// ```
///
/// [`Display`]: ::std::fmt::Display
/// [`to_bytes`]: Self::to_bytes
/// [`ToString`]: ::std::string::ToString
pub trait ToBytes {
    /// Returns a byte string that will be used to represent the instance.
    #[must_use]
    fn to_bytes(&self) -> ByteStr<'_>;

    /// Returns a wide character string that will be used to represent the
    /// instance.
    ///
    /// The Windows API frequently uses wide character strings. This method
    /// allows them to be printed losslessly in some cases, even when they
    /// cannot be converted to UTF-8.
    ///
    /// Returning [`None`] causes [`to_bytes`] to be used instead.
    ///
    /// [`to_bytes`]: Self::to_bytes
    #[cfg(any(doc, windows))]
    #[cfg_attr(print_bytes_docs_rs, doc(cfg(windows)))]
    #[must_use]
    fn to_wide(&self) -> Option<WideStr>;
}

impl ToBytes for [u8] {
    #[inline]
    fn to_bytes(&self) -> ByteStr<'_> {
        ByteStr(ByteStrInner::Bytes(self))
    }

    #[cfg(any(doc, windows))]
    #[inline]
    fn to_wide(&self) -> Option<WideStr> {
        None
    }
}

macro_rules! defer_methods {
    ( $convert_method:ident ) => {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            ToBytes::to_bytes(self.$convert_method())
        }

        #[cfg(any(doc, windows))]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            self.$convert_method().to_wide()
        }
    };
}

impl<const N: usize> ToBytes for [u8; N] {
    defer_methods!(as_slice);
}

impl<T> ToBytes for Cow<'_, T>
where
    T: ?Sized + ToBytes + ToOwned,
    T::Owned: ToBytes,
{
    defer_methods!(deref);
}

macro_rules! defer_impl {
    ( $type:ty , $convert_method:ident ) => {
        impl ToBytes for $type {
            defer_methods!($convert_method);
        }
    };
}
defer_impl!(CStr, to_bytes);
defer_impl!(CString, as_c_str);
defer_impl!(Vec<u8>, as_slice);

#[cfg(any(
    all(target_vendor = "fortanix", target_env = "sgx"),
    target_os = "hermit",
    target_os = "solid_asp3",
    target_os = "wasi",
    target_os = "xous",
    unix,
    windows,
))]
mod os_str {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use super::ByteStr;
    use super::ToBytes;
    #[cfg(any(doc, windows))]
    use super::WideStr;

    impl ToBytes for OsStr {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            #[cfg(windows)]
            {
                use super::ByteStrInner;

                ByteStr(ByteStrInner::Str(self.to_string_lossy()))
            }
            #[cfg(not(windows))]
            {
                #[cfg(all(
                    target_vendor = "fortanix",
                    target_env = "sgx",
                ))]
                use std::os::fortanix_sgx as os;
                #[cfg(target_os = "hermit")]
                use std::os::hermit as os;
                #[cfg(target_os = "solid_asp3")]
                use std::os::solid as os;
                #[cfg(unix)]
                use std::os::unix as os;
                #[cfg(target_os = "wasi")]
                use std::os::wasi as os;
                #[cfg(target_os = "xous")]
                use std::os::xous as os;

                use os::ffi::OsStrExt;

                self.as_bytes().to_bytes()
            }
        }

        #[cfg(any(doc, windows))]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            #[cfg(windows)]
            use std::os::windows::ffi::OsStrExt;

            Some(WideStr(self.encode_wide().collect()))
        }
    }

    defer_impl!(OsString, as_os_str);
    defer_impl!(Path, as_os_str);
    defer_impl!(PathBuf, as_path);
}
