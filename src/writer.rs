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
use std::os::windows::io::AsHandle;

#[cfg(windows)]
use super::console::Console;

pub(super) trait ToConsole {
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

#[cfg(all(feature = "specialization", windows))]
impl<T> ToConsole for T
where
    T: AsHandle + ?Sized + Write,
{
    fn to_console(&self) -> Option<Console<'_>> {
        Console::from_handle(self)
    }
}

/// A bound for [`write_lossy`] that allows it to be used for some types
/// without specialization.
///
/// When the "specialization" feature is enabled, this trait is implemented for
/// all types.
///
/// [`write_lossy`]: super::write_lossy
pub trait WriteLossy {
    #[cfg(windows)]
    #[doc(hidden)]
    fn __to_console(&self) -> Option<Console<'_>>;
}

#[cfg(feature = "specialization")]
#[cfg_attr(print_bytes_docs_rs, doc(cfg(feature = "specialization")))]
impl<T> WriteLossy for T
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
        impl<$generic> WriteLossy for $type
        where
            $generic: ?Sized + WriteLossy,
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
        impl<$generic> WriteLossy for $type
        where
            $generic: Write + WriteLossy,
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

macro_rules! impl_to_console {
    ( $(#[ $attr:meta ])* $type:ty , $to_console_fn:expr , ) => {
        #[cfg(any(doc, not(feature = "specialization")))]
        impl $crate::WriteLossy for $type {
            #[cfg(windows)]
            fn __to_console(&self) -> Option<Console<'_>> {
                $crate::writer::ToConsole::to_console(self)
            }
        }

        $(#[$attr])*
        impl $crate::writer::ToConsole for $type {
            #[cfg(windows)]
            fn to_console<'a>(&'a self) -> Option<Console<'a>> {
                let to_console_fn: fn(&'a Self) -> _ = $to_console_fn;
                to_console_fn(self)
            }
        }
    };
}

macro_rules! r#impl {
    ( $($type:ty),+ ) => {
        $(impl_to_console! {
            #[cfg(not(feature = "specialization"))]
            $type, Console::from_handle,
        })+
    };
}
r#impl!(Stderr, StderrLock<'_>, Stdout, StdoutLock<'_>);

impl_to_console! {
    #[cfg(not(feature = "specialization"))]
    Vec<u8>, |_| None,
}
