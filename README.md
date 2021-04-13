# strict-env

_Parse config values from environment variables_

[![Documentation][docs-badge]][docs-url]
[![Build status][build-badge]][build-url]
[![Test coverage][coverage-badge]][coverage-url]
<br />
[![crates.io][crates-badge]][crates-url]
[![Downloads][downloads-badge]][crates-url]
[![Rust version][rust-version-badge]][rust-version-link]
<br />
[![MIT license][license-badge]][license-url]

[build-badge]: https://img.shields.io/github/workflow/status/andybarron/strict-env-rs/CI?labelColor=112&logo=github&logoColor=fff&style=flat-square
[build-url]: https://github.com/andybarron/strict-env-rs/actions
[coverage-badge]: https://img.shields.io/codecov/c/gh/andybarron/strict-env-rs?labelColor=112&logo=codecov&logoColor=fff&style=flat-square
[coverage-url]: https://codecov.io/gh/andybarron/strict-env-rs
[crates-badge]: https://img.shields.io/crates/v/strict-env?labelColor=112&logo=rust&logoColor=fff&style=flat-square
[crates-url]: https://crates.io/crates/strict-env
[docs-badge]: https://img.shields.io/docsrs/strict-env?labelColor=112&logo=read-the-docs&logoColor=fff&style=flat-square
[docs-url]: https://docs.rs/strict-env
[downloads-badge]: https://img.shields.io/crates/d/strict-env?labelColor=112&color=informational&style=flat-square
[license-badge]: https://img.shields.io/crates/l/strict-env?labelColor=112&style=flat-square
[license-url]: https://github.com/andybarron/strict-env-rs/blob/main/LICENSE.md
[rust-version-badge]: https://img.shields.io/badge/rustc-1.31+-informational?logo=rust&logoColor=fff&labelColor=112&style=flat-square
[rust-version-link]: https://www.rust-lang.org

## Resources

- [**Documentation**][docs-url]
- [crates.io][crates-url]

## TL;DR

```rust
std::env::set_var("PORT", "9001"); // or e.g. dotenv::dotenv()
let port: u16 = strict_env::parse("PORT")?;
assert_eq!(port, 9001);
```
