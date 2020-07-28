`wxrust_sys` is meant to just be raw FFI bindings to wxWidgets. The build script
automatically searches for a suitable `wx-config` and draws all configuration
from there, except for include paths (right now).

Currently, it's nonfunctional because `bindgen` is dumb and can't generate Rust
code effectively. You need [a
hack](https://github.com/rust-lang/rust-bindgen/issues/1834#issuecomment-664120228)
in order to even run the build script on macOS, and the generated code doesn't
even compile because of
[duplicate definitions](https://github.com/rust-lang/rust-bindgen/issues/1848).

At least until the code compiles, I can't work on this. wxWidgets is a humongous
library with hundreds of thousands of lines of header files weighing in at
many megabytes. There's no way I'd be able to do the bindings manually.
