# Providers

Starkli connects to Starknet through "providers". Many commands require a provider to be supplied.

Previously, Starkli supported using both JSON-RPC and the sequencer gateway for accessing the network. However, following the [deprecation of the sequencer gateway](https://community.starknet.io/t/feeder-gateway-deprecation/100233), support for using the sequencer gateway has been dropped. Therefore, the only provider supported now is JSON-RPC.

There are two ways to specify a JSON-RPC provider, either [directly](#using-an-rpc-url-directly) or through [predefined networks](#using-a-predefined-network).

> 💡 **Tips**
>
> Each Starkli version only works with one specific JSON-RPC specification version. To check the supported JSON-RPC version, run the verbose version output command:
>
> ```console
> starkli -vV
> ```

> ℹ️ **Note**
>
> When no provider option is supplied, Starkli falls back to using the `goerli` network. If the network is not already defined, a [free RPC vendor](#free-rpc-vendors) is used.
>
> You're advised against relying on the fallback to use the `goerli` network, as the default network might change over time. Therefore, a warning is shown each time the fallback is used.

## Using an RPC URL directly

There are a few options to obtain access to a JSON-RPC endpoint:

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
> While using `--rpc` or `STARKNET_RPC` is convenient for one-off command invocations, using [predefined networks](#using-a-predefined-network) is recommended for more complicated use cases.

## Using a predefined network

Networks can be defined in [profiles](./profiles.md). Each network is uniquely identified by an identifier within a profile. When the `--network` option, or the `STARKNET_NETWORK` environment variable, is used, Starkli looks up the network identifier in the current active profile, and uses its provider settings to connect to the network. See the [profiles page](./profiles.md) for details on defining networks.

If the supplied network identifier is not found, Starkli terminates with an error, **unless the network is eligible for [free RPC vendors](#free-rpc-vendors)**, in which case Starkli automatically creates the network and persists it into the profile.

For example, to check the block height of a predefined network `mainnet`:

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

> ℹ️ **Note**
>
> `--rpc` or `STARKNET_RPC` take precedence over `--network` or `STARKNET_NETWORK`. When both options are supplied, `--network` (`STARKNET_NETWORK`) is ignored, and a warning message is shown.

### Free RPC vendors

Historically, the now-deprecated-and-removed sequencer gateway provider allowed new Starkli users to start interacting with Starknet without going through the hassle of obtaining a JSON-RPC endpoint. However, following the [deprecation of the sequencer gateway](https://community.starknet.io/t/feeder-gateway-deprecation/100233), this is no longer an option. To maintain the same zero-setup experience, support for free RPC vendors was added.

The following 3 networks are eligible for free RPC vendors:

- `mainnet`
- `goerli`
- `sepolia`

When using these networks, **and when the network is not already defined in the active profile**, a free vendor will be randomly chosen from below:

- [Blast](https://blastapi.io/public-api/starknet)
- [Nethermind](https://data.voyager.online/)

Once selected, the vendor choice is persisted in the profile. A message is printed to the console when this happens. All subsequent invocations under the same network use the already chosen vendor automatically.

> 💡 **Tips**
>
> You can always change the automatically assigned free RPC vendor for a network by [editing the profiles](./profiles.md).
