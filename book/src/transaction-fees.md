# Transaction fees

Historically, Starknet transaction fees could be paid with either _ETH_ or _STRK_, as indicated by the transaction version used. However, since Starknet JSON-RPC spec version v0.8.0, the ability to send pre-v3 transactions (and thus pay fees with _ETH_) had been dropped. As a result, Starkli also no longer supports paying fees in _ETH_ starting with release v0.4.0, where Starkli switched from spec v0.7.1 to v0.8.0.

> ℹ️ **Note**
>
> While JSON-RPC spec v0.8.0 dropped support for pre-v3 transactions, as of this writing they are still accepted by the Starknet network itself. If you need to send pre-v3 transactions, use a v0.3.x Starkli release.
>
> This could be useful you are working with certain contracts that revert on v3 transactions. Do note that it's recommended that you upgrade these contracts to be v3-compatible, as pre-v3 transactions will eventually be disabled on the network itself.

## Setting transaction fees manually

By default, Starkli requests a fee estimate from the [provider](./providers.md), and a 50% buffer is added on top of the estimate to avoid failures due to price fluctuations. However, sometimes it might be desirable to manually specify the fees instead. Some common reasons to do this include:

- The fee estimation returned by the provider is inaccurate;
- You want to speed up command execution by skipping fee estimation; or
- The transaction in question is flaky, so the estimation might fail.

### Setting _STRK_ fees manually

In transactions that pay fees in _STRK_, the fee is made up of 6 components:

- `l1_gas`
- `l1_gas_price`
- `l2_gas`
- `l2_gas_price`
- `l1_data_gas`
- `l1_data_gas_price`

These components indicate the maximum amount a user is willing to pay for a specific resource.

Each component has its own command line option for setting a manual value to be used:

- `l1_gas`: `--l1-gas`
- `l1_gas_price`: `--l1-gas-price` or `--l1-gas-price-raw`
- `l2_gas`: `--l2-gas`
- `l2_gas_price`: `--l2-gas-price` or `--l2-gas-price-raw`
- `l1_data_gas`: `--l1-data-gas`
- `l1_data_gas_price`: `--l1-data-gas-price` or `--l1-data-gas-price-raw`

To set any of the `gas` values, simply use its corresponding option. For example, to manually set a `l1_gas` value of `50000`:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --l1-gas 50000
```

> 💡 **Tips**
>
> When _all_ 3 `gas` options are manually set, Starkli does _not_ perform a fee estimation and instead sources the `gas_price` values directly from the pending block header. Therefore, transaction failure will _not_ be detected until it's reverted on-chain.

To set any of the `gas_price` values, use any of its two options. The `--xx-gas-price` variant accepts a decimal value in _STRK_ (18 decimals). For example, to set the L1 gas price at `0.0001` _STRK_:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --l1-gas-price 0.0001
```

Alternatively, the `‐‐xx-gas-price-raw` variant can be used for raw price values in _FRI_ (the smallest unit of _STRK_). This command is equivalent to the one shown above:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --l1-gas-price-raw 100000000000000
```

> 💡 **Tips**
>
> Even when all `gas_price` options are manually set, as long as _any_ of the `gas` options is not set, Starkli will still perform a fee estimation to determine how much gas is needed.

## Estimating the fee only (dry run)

Commands that send out transactions accept a `--estimate-only` flag, which stops command execution as soon as an estimate is generated.

An example command to estimate the fee for an _ETH_ transfer:

```console
starkli invoke eth transfer 0x1234 u256:100 --estimate-only
```
