# print\_bytes

This crate allows printing broken UTF-8 bytes to an output stream as losslessly
as possible.

Usually, paths are printed by calling [`Path::display`] or
[`Path::to_string_lossy`] beforehand. However, both of these methods are always
lossy; they misrepresent some valid paths in output. The same is true when
using [`String::from_utf8_lossy`] to print any other UTF-8–like byte sequence.

Instead, this crate only performs a lossy conversion when the output device is
known to require Unicode, to make output as accurate as possible. When
necessary, any character sequence that cannot be represented will be replaced
with [`REPLACEMENT_CHARACTER`]. That convention is shared with the standard
library, which uses the same character for its lossy conversion functions.

[![GitHub Build Status](https://github.com/dylni/print_bytes/workflows/build/badge.svg?branch=master)](https://github.com/dylni/print_bytes/actions?query=branch%3Amaster)

## Usage

Add the following lines to your "Cargo.toml" file:

```toml
[dependencies]
print_bytes = "1.2"
```

See the [documentation] for available functionality and examples.

## Rust version support

The minimum supported Rust toolchain version is currently Rust 1.63.0.

Minor version updates may increase this version requirement. However, the
previous two Rust releases will always be supported. If the minimum Rust
version must not be increased, use a tilde requirement to prevent updating this
crate's minor version:

```toml
[dependencies]
print_bytes = "~1.2"
```

## License

Licensing terms are specified in [COPYRIGHT].

Unless you explicitly state otherwise, any contribution submitted for inclusion
in this crate, as defined in [LICENSE-APACHE], shall be licensed according to
[COPYRIGHT], without any additional terms or conditions.

[COPYRIGHT]: https://github.com/dylni/print_bytes/blob/master/COPYRIGHT
[documentation]: https://docs.rs/print_bytes
[LICENSE-APACHE]: https://github.com/dylni/print_bytes/blob/master/LICENSE-APACHE
[`Path::display`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.display
[`Path::to_string_lossy`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.to_string_lossy
[`REPLACEMENT_CHARACTER`]: https://doc.rust-lang.org/std/char/constant.REPLACEMENT_CHARACTER.html
[`String::from_utf8_lossy`]: https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy
