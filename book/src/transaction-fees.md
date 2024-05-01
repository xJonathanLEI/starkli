# Transaction fees

Historically, Starknet transaction fees could only be paid with _ETH_. With the introduction of [v3 transactions](https://github.com/starknet-io/SNIPs/blob/main/SNIPS/snip-8.md), however, users now have the option to pay fees in _STRK_ if they so choose.

By default, Starkli sends transactions with _ETH_ fees. The fee token can be set by using the `--fee-token` option (e.g., `--fee-token STRK`). Alternatively, you can also use the `--eth` or `--strk` shorthands.

> ℹ️ **Note**
>
> While it might seem to be the case, it's not true that v3 transactions provide the option to choose between fee tokens. Instead, v3 transactions are _always_ paid with _STRK_. Starkli abstracts this away by automatically falling back to using older transaction versions when users choose to pay fees in _ETH_.

## Setting transaction fees manually

By default, Starkli requests a fee estimate from the [provider](./providers.md), and a 50% buffer is added on top of the estimate to avoid failures due to price fluctuations. However, sometimes it might be desirable to manually specify the fees instead. Some common reasons to do this include:

- The fee estimation returned by the provider is inaccurate;
- You want to speed up command execution by skipping fee estimation; or
- The transaction in question is flaky, so the estimation might fail.

Since transactions paying in _ETH_ and _STRK_ are priced differently, the options for manually setting fees are different depending on which token you're paying fees with.

### Setting _ETH_ fees manually

Transactions that pay fees in _ETH_ are priced using a single `max_fee` value, which indicates the maximum amount of fees (in `Wei`) that an account is willing to pay.

Users can set the `max_fee` value by using the `--max-fee` option, which accepts a decimal value in Ether (18 decimals). For example, to perform an _ETH_ transfer with a `max_fee` of `0.01 ETH`:

```console
starkli invoke eth transfer 0x1234 u256:100 --max-fee 0.01
```

If you already have the `max_fee` value in `Wei`, you can also use the raw value directly via the `--max-fee-raw` option. An equivalent command to the example above would be:

```console
starkli invoke eth transfer 0x1234 u256:100 --max-fee-raw 10000000000000000
```

### Setting _STRK_ fees manually

Transactions that pay fees in _STRK_ are priced differently from those that pay with _ETH_. Specifically, the fee is made up of two components: `gas` and `gas_price`, which indicate the maximum amount of **L1 gas** and the maximum **L1 gas price** a user is willing to pay, respectively.

To set the `gas` value, simply use the `--gas` option. For example, to manually set a `gas` value of `50000`:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --gas 50000
```

> 💡 **Tips**
>
> When `gas` is manually set but `gas_price` is not, Starkli does _not_ perform a fee estimation and instead sources the `gas_price` value directly from the pending block header. Therefore, transaction failure will _not_ be detected until it's reverted on-chain.

To set the `gas_price` value, use the `--gas-price` option, which accepts a decimal value in _STRK_ (18 decimals). For example, to set the L1 gas price at `0.0001` _STRK_:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --gas-price 0.0001
```

Similar to setting `max_fee` for _ETH_-priced transactions, the `gas_price` value can also be set with raw values by using `--gas-price-raw`. This command is equivalent to the one shown above:

```console
starkli invoke --strk eth transfer 0x1234 u256:100 --gas-price-raw 100000000000000
```

> 💡 **Tips**
>
> When `gas_price` is manually set but `gas` is not, Starkli will still perform a fee estimation to determine how much L1 gas is needed.

## Estimating the fee only (dry run)

Commands that send out transactions accept a `--estimate-only` flag, which stops command execution as soon as an estimate is generated.

An example command to estimate the fee for an _ETH_ transfer:

```console
starkli invoke eth transfer 0x1234 u256:100 --estimate-only
```
