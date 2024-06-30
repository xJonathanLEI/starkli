# EIP-2645 wallet paths

Ledger derives its private keys based on [Hierarchical Deterministic Wallets derivation paths](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) (HD paths), allowing a single Ledger device to use an unlimited number of keys. HD paths can be thought of as IDs of keys. The same combination of Ledger seed phrase + path would always result in the same key pair.

Therefore, it's important to decide on, and **document** the HD paths to use for your accounts. While it's trivial to use a keypair given an HD path, the reverse is not true. It could be extremely difficult to figure out which HD path corresonds to a certain public key. It's therefore recommended that you follow [established patterns of path management](#path-management-best-practices) (e.g. using incremental numbers staring from zero), and document the paths used.

## The EIP-2645 standard

If you've used hardware wallets in the past, you might already be familiar with the concept of HD paths, which might look like this: `m/44'/60'/0'/0/0`. Notably, this path would _not_ work with the Starknet Ledger app, as the app _only works with a specific path format known as [EIP-2645 HD paths](https://github.com/ethereum/ercs/blob/master/ERCS/erc-2645.md)_.

An EIP-2645 takes the format of:

```
m/2645'/layer'/application'/eth_address_1'/eth_address_2'/index
```

where `layer`, `application`, `eth_address_1`, `eth_address_2`, and `index` are 31-bit unsigned numbers.

## The Starkli extension

EIP-2645 paths are difficult to write (e.g. `m/2645'/1195502025'/1470455285'/0'/0'/0`). Therefore, Starkli provides an extension to the standard to enhance the ease of use.

> ℹ️ **Note**
>
> These additional convenience are a _Starkli-only_ extension, and paths written this way would not work with other HD wallet software (including `starknet-rs`). To obtain a standard, universal path representation, Starkli provides the `starkli eip2645 echo` command:
>
> ```console
> starkli eip2645 echo "m//starknet'/starkli'/0'/0'/5"
> ```
>
> which would output `m/2645'/1195502025'/1470455285'/0'/0'/5`, a path representation that's universally accepted. It's also apparent how Starkli's format is much more user-friendly.

### Using non-numerical strings

As per the EIP-2645 standard, `layer` and `application` are defined as hashes of the names of `layer` and `application`. Instead of having to manually calculate the hashes and writing error-prone numbers, Starkli provides an extension to the standard that allows users to fill in non-numerical strings, which would be automatically converted into numbers.

An example of such a path under the Starkli extension would be:

```
m/2645'/starknet'/starkli'/0'/0'/0
```

### Omitting the `2645'` segment

Since Starkli only works with EIP-2645 paths, and these paths always start with the first level as `2645'`, Starkli allows leaving the first level empty, like so:

```
m//starknet'/starkli'/0'/0'/0
```

## Path management best practices

The single most important rule of path management is to **document the paths you've used**, especially if you use exotic paths that are not typically used (and hence key discovery algorithms wouldn't find them).

In terms of deciding on path levels, a widely agreed-upon path management pattern has not been established. Starkli recommends:

1. always using the `m//starknet'/starkli'` prefix for the first 3 levels;
2. keeping the `eth_address_1` level constant at `0'`;
3. starting the `eth_address_2` level from `0'` for your first account, and incrementing or more accounts;
4. keeping the `index` level at `0`.

> ℹ️ **Note**
>
> Some might suggest that it's best to keep both `eth_address_1` and `eth_address_2` at `0'` and only ever increment `index`. However, doing so has a (very small) security risk of having the entire set of keys compromised if:
>
> - any private key in the set is compromised; **and**
> - the attacker has access to the `xpub` key on the `eth_address_2` level.
>
> While the risk is rather small, Starkli would still recommend using `eth_address_2` as the counter instead.

> ℹ️ **Note**
>
> In the original EIP-2645 standard, the levels `eth_address_1` and `eth_address_2` are for identifying the layer-1 address, assuming a layer-2 user would always start with some layer-1 account.
>
> This might be true for services like StarkEx, but is apparently not applicable to Starknet. Therefore, the original purpose of these 2 levels are discarded here.
