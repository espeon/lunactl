# lunactl

A CLI tool to manage [TidaLuna](https://github.com/inrixia/tidaluna) on your system. Not affiliated.

## Installation

### Download a binary

Download from GitHub Releases: [lunactl](https://github.com/espeon/lunactl/releases/)

### Install via cargo

`cargo install --git https://github.com/espeon/lunactl`

> [!IMPORTANT]
> On macOS, if you download **lunactl** from the internet, you may need to run the following command to allow it to run:
> ```bash
> xattr -d com.apple.quarantine ./lunactl
> ```
> For more details, visit the [Apple Support KB entry](https://support.apple.com/en-us/102445).

### Build manually

You'll need Rust and Cargo installed (ideally via rustup).

1. Clone via `git clone https://github.com/espeon/lunactl`
2. Run `cargo build --release`
3. Optionally install with `cargo install --path .`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE/LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your discretion.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
