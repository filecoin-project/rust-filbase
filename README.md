# Filbase

[![CircleCI][circleci-shield]][circleci] [![License][license-shield]][license]

> Filecoin proofs & sector management in a convenient package.


**Warning**: Requires a _new_ rust nightly.


## Building

```sh
> cargo build --release
```

In case you have errors during the build try to update your nightly version:

```sh
rustup update && rustup toolchain install nightly && cargo build --release
```

## Usage

```sh
# Start the daemon
> filbase daemon

# In another terminal
> filbase sector size
  1024
```

## Benchmarks

In order to use this tool to run benchmarks, it needs to be compiled with the `benchy` feature.

```sh
> cargo build --release --features benchy
> ./target/release/filbase benchy --help
```


## Testing

```sh
> cargo test
```

## License

The Filecoin Project is dual-licensed under Apache 2.0 and MIT terms:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)


[circleci-shield]: https://img.shields.io/circleci/project/github/filecoin-project/rust-filbase.svg?style=flat-square
[circleci]: https://circleci.com/gh/filecoin-project/rust-filbase
[license-shield]: https://img.shields.io/badge/License-MIT%2FApache2.0-green.svg?style=flat-square
[license]: https://github.com/dignifiedquire/rust-accumulators/blob/master/LICENSE.md
