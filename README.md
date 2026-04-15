# [`fre-rs`](https://crates.io/crates/fre-rs)

Safe, ergonomic Rust abstraction over the AIR Native Extension (ANE) C API ([`fre-sys`](https://crates.io/crates/fre-sys)) for native-side development.

This crate depends on the AIR SDK via [`fre-sys`](https://crates.io/crates/fre-sys). See its documentation for setup instructions.

## References

- [Adobe AIR SDK from HARMAN](https://airsdk.harman.com)
- [AIR | AIR SDK](https://airsdk.dev/docs)
- [ActionScript 3.0 Language Reference](https://airsdk.dev/reference/actionscript/3.0)
- [Adobe Flash Platform * Developing Native Extensions for AdobeAIR](https://help.adobe.com/en_US/air/extensions/index.html)
- [Adobe ActionScript® 3 (AS3) API Reference](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/index.html)

## Safety

This crate provides safe abstractions over the ANE C API.

Underlying objects may still be modified externally by the AIR runtime.

Unsafe code is minimized and encapsulated, but correct usage of this crate's API is still required.

## License

MIT OR Apache-2.0

## Getting Started

The primary entry points of this crate are the macros [`extension!`](https://docs.rs/fre-rs/latest/fre_rs/macro.extension.html) and [`function!`](https://docs.rs/fre-rs/latest/fre_rs/macro.function.html).

Refer to their documentation for details and examples.

## Example

```rust
use fre_rs::prelude::*;
fre_rs::extension! {
    extern Initializer;
    gen init_ctx, final;
}
fn init_ctx(_: &CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet) {
    let mut funcs = FunctionSet::new();
    funcs.add(None, None, hello);
    (None, funcs)
}
fre_rs::function! {
    hello (ctx, _, args) -> as3::String {
        ctx.trace(args);
        as3::String::new(ctx, "Hello! Flash Runtime.")
    }
}
```

The repository provides a [comprehensive example](https://github.com/frashack/fre-rs/tree/main/examples/windows-x86-64) of ANE integration within an AIR project.
It covers the full development workflow, including native implementation (Rust),
ActionScript 3.0 wrapper, and automated build script.

To build and execute the example, ensure the following environment is configured:

- Operating System: Windows 10+ (x86-64)
- Rust Toolchain: version 1.85+ (supporting Edition 2024)
- C++ Build Tools: Visual Studio Build Tools (MSVC)
- AIR SDK: HARMAN AIR SDK (latest version recommended)
- Java Runtime: JRE or JDK 8+ (required for the AIR Developer Tool)