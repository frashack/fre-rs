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

## Example

```rust
use fre_rs::prelude::*;
fre_rs::extension! {
    extern symbol_initializer;
    gen context_initializer, final;
}
fn context_initializer(frt: &FlashRuntime) -> FunctionSet {
    let mut funcs = FunctionSet::new();
    funcs.add(None, None::<()>, method_name);
    funcs
}
fre_rs::function! {
    method_name (frt, _, args) -> Str {
        frt.trace(args);
        Str::new(frt, "Hello! Flash Runtime")
    }
}
```