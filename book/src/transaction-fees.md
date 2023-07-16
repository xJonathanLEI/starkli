# Transaction fees

Starknet transactions are priced using a single `max_fee` value, which indicates the maximum amount of fees (in `Wei`) that an account is willing to pay.

For commands that send out transactions, Starkli needs to come up with this value. By default, a fee estimate is requested from the [provider](./providers.md), and a 50% buffer is added on top of the estimate to avoid failures due to price fluctuations.

## Setting `max_fee` manually

It's possible to skip the entire fee estimation process by manually providing a `max_fee` value.

The recommended way to do it is through the `--max-fee` option, which accepts a decimal value in Ether (18 decimals). For example, to perform an `ETH` transfer with a `max_fee` of `0.01 ETH`:

```console
starkli invoke eth transfer 0x1234 u256:100 --max-fee 0.01
```

If you already have the `max_fee` value in `Wei`, it's also possible to use the raw value directly via the `--max-fee-raw` option. An equivalent command to the example above would be:

```console
starkli invoke eth transfer 0x1234 u256:100 --max-fee-raw 10000000000000000
```

## Estimating fee only (dry run)

Commands that send out transactions accept a `--estimate-only` flag, which stops command execution as soon as an estimate is generated.

An example command to estimate the fee for an `ETH` transfer:

```console
starkli invoke eth transfer 0x1234 u256:100 --estimate-only
```
