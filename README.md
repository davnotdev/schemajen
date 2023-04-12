# SchemaJen

Auto-magically infer language bindings given a JSON schema.

Bridging client and backend is hard.
This is especially true if your backend is written in a different language from your client.
Schemajen is a tool that makes the process almost painless.
Given a JSON request, response, or literally anything, SchemaJen can infer working language bindings.

## Try it now!

You can try the web version of SchemaJen [here](https://davnotdev.github.io/schemajen);

Alternatively, you can also install the CLI version of SchemaJen using cargo.

`cargo install schemajen`

## Contributing / Using the Crate

Being built in Rust, SchemaJen comes as a crate.
You can find more on the [docs](https://docs.rs/schemajen) and [crate information](https://crates.io/crates/schemajen) pages.

### Example Usage

```rust
use schemajen::*;

//  See [`ACCUMULATOR_SUPPORT_LIST`] for string options.
//  let mut accumulator = accumulator_choose_with_str("rust");

let mut accumulator = Box::new(RustAccumulator::begin());
let res = generate(&mut accumulator, "MyType", r#"{"a": 10}"#);
res.unwrap();
eprintln!("{}", res);

```
