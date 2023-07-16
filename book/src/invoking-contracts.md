# Invoking contracts

With Starkli, this is done with the `starkli invoke` command.

> ℹ️ **Note**
>
> You need both a [signer](./signers.md) and an [account](./accounts.md) for this. The commands shown in this page omit the signer and account options for better readability, and assume you've properly configured the environment variables.

The basic format of a `starkli invoke` command is the following:

```console
starkli invoke <ADDRESS> <SELECTOR> <ARGS>
```

For example, to transfer `100 Wei` of the `ETH` token to the address `0x1234`, one can run:

```console
starkli invoke 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7 transfer 0x1234 100 0
```

> ℹ️ **Note**
>
> The `transfer` function takes 2 parameters (i.e. `recipient` and `amount`) but we actually need to enter 3 (note the `0` at the end). This is because `amount` is of type `u256`, which consists of 2 raw field elements.
>
> See the [simplifying invoke commands](#simplifying-invoke-commands) section below for ways to make entering this command easier.

## Simplifying invoke commands

You might be able to simplify invoke commands by leveraging [argument resolution](./argument-resolution.md). In this section, we will take the `ETH` transfer command above and try to simplify it.

First, since the `0x021074834d251687180a8d007c5ffc5819e3e68993de9d2d2c32a67d9f3091ff` address is a well-known address available on the built-in address book as the `eth` entry, we can replace it with the use of the [`addr` scheme](./argument-resolution.md#addr):

```console
starkli invoke addr:eth transfer 0x1234 100 0
```

Furthermore, as the `addr:eth` is the first positional argument in an `invoke` command, it's eligible for [scheme omission](./argument-resolution.md#scheme-omission), which means we can further simplify it by dropping the `addr:` prefix:

```console
starkli invoke eth transfer 0x1234 100 0
```

Manually entering `u256` values as 2 separate field element values is tedious and error-prone, especially with larger values. We can leverage the [`u256` scheme](./argument-resolution.md#u256) to have Starkli automatically split the values for us:

```console
starkli invoke eth transfer 0x1234 u256:100
```

For more information regarding argument resolution, check out the [argument resolution](./argument-resolution.md) page.

## Multicall support

Starkli has seamless support for multicall. To use more than 1 contract call in an `invoke` command, simply separate the calls with `/`.

For example, to also approve the sending of `300 Wei` for address `0x4321` in the same transaction:

```console
starkli invoke eth transfer 0x1234 u256:100 / eth approve 0x4321 u256:300
```
