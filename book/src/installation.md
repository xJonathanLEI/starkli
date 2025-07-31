# Installation

The easiest way to install Starkli is to use `starkliup`, a portable script that downloads prebuilt binaries and manages shell configuration for you. However, it might not be available depending on your device's platform and/or architecture.

Note that if you use any installation method other than `starkliup`, you will need to manually [set up shell completions](./shell-completions.md).

## Using `starkliup`

If you're on Linux/macOS/WSL/Android, you can install `starkliup` by running the following command:

```console
curl https://get.starkli.sh | sh
```

You might need to restart your shell session for the `starkliup` command to become available. Once it's available, run the `starkliup` command:

```console
starkliup
```

Running the commands installs `starkli` for you, and upgrades it to the latest release if it's already installed.

`starkliup` detects your device's platform and automatically downloads the right prebuilt binary. It also sets up shell completions. You might need to restart your shell session for the completions to start working.

> ℹ️ **Note**
>
> Over time, `starkliup` itself may change and require upgrading. To upgrade `starkliup` itself, run the `curl` command above again.

## Prebuilt binaries

Prebuilt binaries are available with [GitHub releases](https://github.com/xJonathanLEI/starkli/releases) for certain platforms.

Prebuilt binaries are best managed with [`starkliup`](#using-starkliup). However, if you're on a platform where `starkliup` isn't available (e.g. using `starkli` on Windows natively), you can manually download the prebuilt binaries and make them available from `PATH`.

## Install from source

If you have [Rust](https://www.rust-lang.org/) installed, it's also possible to install `starkli` directly from source. Installing from source might be necessary if you want to use an unreleased feature, for example.

> ℹ️ **Note**
>
> Shell completions would _not_ be configured when you install `starkli` from source. You need to manually [set up shell completions](./shell-completions.md) for it to work.

To install from [GitHub](https://github.com/xJonathanLEI/starkli):

```console
cargo install --locked --git https://github.com/xJonathanLEI/starkli
```

> ℹ️ **Note**
>
> It's not recommended to install Starkli from [crates.io](https://crates.io/), as Starkli is no longer published there there after v0.1.8.
>
> This is because Starkli uses Git dependencies due to the need to bundle multiple SemVer-compatible versions of the Sierra compiler.

## Install via asdf

[asdf](ttps://asdf-vm.comttps://asdf-vm.com) is a CLI tool that can manage multiple language runtime versions on a per-project basis.

- Run the following to add the `starkli` plugin
```console
asdf plugin add starkli
```
- Show all installable versions:
```console
asdf list-all starkli
```
- Install latest version:
```console
asdf install starkli latest
```
- Install specific version:
```console
asdf install starkli 0.3.5
```

Check [asdf guide](https://asdf-vm.com/guide/getting-started.html) for more instructions on how to install & manage versions.