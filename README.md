<p align="center">
  <h1 align="center">starkli</h1>
</p>

**Starkli (/ˈstɑːrklaɪ/), a :zap: blazing :zap: fast :zap: CLI tool for StarkNet powered by :crab: [starknet-rs](https://github.com/xJonathanLEI/starknet-rs) :crab:**

## Installation

The package will be published to crates.io when it's more feature-complete. For now, install from GitHub directly for the latest features and bug fixes:

```sh
$ cargo install --locked --git https://github.com/xJonathanLEI/starkli
```

## Commands

Check the list of available commands by simply running `starkli` without arguments:

```console
$ starkli
Starkli (/ˈstɑːrklaɪ/), a blazing fast CLI tool for StarkNet powered by starknet-rs

Usage: starkli <COMMAND>

Commands:
  selector                 Calculate selector from name
  class-hash               Calculate class hash from compiled contract artifact
  to-cairo-string          Encode string into felt with the Cairo short string representation
  parse-cairo-string       Decode string from felt with the Cairo short string representation
  get-transaction          Get StarkNet transaction by hash
  block-number             Get latest block number
  get-block                Get StarkNet block
  block-time               Get StarkNet block timestamp only
  get-transaction-receipt  Get transaction receipt by hash
  chain-id                 Get StarkNet network ID
  completions              Generate shell completions script
  help                     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
