## Install

```shell
$ cargo install --git https://github.com/emi2k01/cargo-check-tip.git
```

## Usage

```shell
$ cargo check-tip
```

### For library developers

In your `Cargo.toml`

```toml
[package.metadata.tips]
tip = "Functions must be async"
code_pattern = "(E0456|E0987)" # Rust code error pattern
message_pattern = "the trait bound.*" # Error message pattern
span_pattern = "the trait bound.*" # Span pattern, used to mark where the tip will point to
```

#### Rust error anatomy
![Error anatomy](https://user-images.githubusercontent.com/78516649/215917091-c792874d-755e-4893-bc1d-f661f2584675.png)
