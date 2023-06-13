<p align="center">
  <h1 align="center">starkli</h1>
</p>

**Starkli (/ˈstɑːrklaɪ/), a :zap: blazing :zap: fast :zap: CLI tool for Starknet powered by :crab: [starknet-rs](https://github.com/xJonathanLEI/starknet-rs) :crab:**

## Installation

The package will be published to crates.io when it's more feature-complete. For now, install from GitHub directly for the latest features and bug fixes:

```sh
$ cargo install --locked --git https://github.com/xJonathanLEI/starkli
```

## Commands

Check the list of available commands by simply running `starkli` without arguments:

```console
$ starkli
Starkli (/ˈstɑːrklaɪ/), a blazing fast CLI tool for Starknet powered by starknet-rs

Usage: starkli <COMMAND>

Commands:
  selector             Calculate selector from name
  class-hash           Calculate class hash from any contract artifacts (Sierra, casm, legacy)
  to-cairo-string      Encode string into felt with the Cairo short string representation
  parse-cairo-string   Decode string from felt with the Cairo short string representation
  mont                 Print the montgomery representation of a field element
  call                 Call contract functions without sending transactions
  transaction          Get Starknet transaction by hash
  block-number         Get latest block number
  block-hash           Get latest block hash
  block                Get Starknet block
  block-time           Get Starknet block timestamp only
  state-update         Get state update from a certain block
  transaction-receipt  Get transaction receipt by hash
  chain-id             Get Starknet network ID
  nonce                Get nonce for a certain contract
  storage              Get storage value for a slot at a contract
  class-hash-at        Get contract class hash deployed at a certain address
  class-by-hash        Get contract class by hash
  class-at             Get contract class deployed at a certain address
  syncing              Get node syncing status
  signer               Signer management commands
  account              Account management commands
  declare              Declare a contract class
  deploy               Deploy contract via the Universal Deployer Contract
  completions          Generate shell completions script
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
