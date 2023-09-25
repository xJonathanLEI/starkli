# Providers

Starkli connects to Starknet through "providers". Many commands require a provider to be supplied.

Currently, two providers are supported: [JSON-RPC](#json-rpc) and the [sequencer gateway](#sequencer-gateway) (deprecated).

> ℹ️ **Note**
>
> When no provider option is supplied, Starkli falls back to using the sequencer gateway provider for the `goerli-1` network.

## JSON-RPC

Starkli is centric around JSON-RPC, and the JSON-RPC provider is considered canonical. Users are strongly recommended to use JSON-RPC. There are a few options to obtain access to a JSON-RPC endpoint:

- hosting your own node with [`pathfinder`](https://github.com/eqlabs/pathfinder) or [`juno`](https://github.com/NethermindEth/juno); or
- using a third-party JSON-RPC API provider like [Infura](https://www.infura.io/), [Alchemy](https://www.alchemy.com/), [Chainstack](https://chainstack.com/build-better-with-starknet/), or [Nethermind](https://starknetrpc.nethermind.io/).

Once you have a URL to a JSON-RPC endpoint, you can use it via the `--rpc <URL>` option for commands that expect it. For example:

```console
starkli block-number --rpc http://localhost:9545/
```

Alternatively, you can set the `STARKNET_RPC` environment variable to make command invocations easier:

```console
export STARKNET_RPC="http://localhost:9545/"
```

and then, simply run:

```console
starkli block-number
```

which is the same as the running with the `--rpc` option.

> 💡 **Tips**
>
> Each Starkli version only works with one specific JSON-RPC specification version. To check the supported JSON-RPC version, run the verbose version output command:
>
> ```console
> starkli -vV
> ```

## Sequencer gateway

> ⚠️ **Warning**
>
> The sequencer gateway is deprecated and will be disabled by StarkWare soon. You're strongly recommended to use the [JSON-RPC provider](#json-rpc) instead.

Historically, before the JSON-RPC API became available, access to the network had been possible only through a set of API offered by StarkWare known as the sequencer gateway. As of this writing, despite the wide availability of the JSON-RPC API, StarkWare still runs the sequencer gateway, but has declared it as deprecated, and encourages users to migrate to use JSON-RPC instead.

> ℹ️ **Note**
>
> To raise awareness of the deprecation, Starkli always displays a warning message when the sequencer gateway provider is used.

To use the sequencer gateway anyways, use the `--network <NETWORK>` option, where `<NETWORK>` is one of the following:

- `mainnet`
- `goerli-1`
- `goerli-2`
- `integration`

For example, to check the latest block number on `mainnet`:

```console
starkli block-number --network mainnet
```

Alternatively, you can set the `STARKNET_NETWORK` environment variable to make command invocations easier:

```console
export STARKNET_NETWORK="mainnet"
```

and then, simply run:

```console
starkli block-number
```

which is the same as the running with the `--network` option.
