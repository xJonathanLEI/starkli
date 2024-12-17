# Argument resolution

To make argument input easier and less error-prone, Starkli supports _argument resolution_, the process of expanding simple, human-readable input into actual field element arguments expected by the network.

By default, arguments are only parsed as raw field elements, accepting both decimal and hexadecimal representations.

To make arguments expandable, and thus trigger the resolution process, use the format `scheme:content`, where `scheme` is one of the supported [schemes](#schemes), and `content` is the scheme-specific content.

## Schemes

### `addr`

The `addr` scheme resolves the address name provided as `content` into a full address using an _address book_ under the current network ID. As of this writing, the actual address book feature hasn't been implemented, and a hard-coded address book is used instead, which contains only one entry `eth` for the `ETH` token address.

### `u256`

The `u256` scheme interprets `content` as an unsigned 256-bit integer and resolves into _2_ field element arguments for the low and high 128 bits, respectively. This scheme is useful for working with contracts expecting `u256` arguments, such as the standard ERC20 contract.

### `str`

The `str` scheme encodes `content` as [Cairo short string](https://book.starknet.io/chapter_2/strings.html#working_with_short_strings).

### `const`

The `const` scheme uses `content` as the key to look up a hard-coded table to commonly used constant values. The current list of constants are:

| Key        | Value                                                                    |
| ---------- | ------------------------------------------------------------------------ |
| `u256_max` | `0xffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffff` |
| `felt_max` | `0x0800000000000011000000000000000000000000000000000000000000000000`     |

### `selector`

The `selector` scheme calculates the _Starknet Keccak_ hash for the content to derive the function entryponit.

### `storage`

This scheme is currently the same as `selector`, but support for offsets and maps (e.g. `ERC20_balances[0x1234]`) might be added in the future to differentiate it.

### `bytearray`

The `bytearray` scheme encodes a list of bytes in the format of the `ByteArray` Cairo type.

The simplest use of this scheme is by supplying a hexadecimal representation of the raw bytes. Example: `bytearray:0x1234`.

Since a common use of the `ByteArray` type is to encode strings, the `bytearray` scheme has support for that too. To use it, simply prepend `str:` to the value. For example, `bytearray:str:hello` is equivalent to `bytearray:0x68656c6c6f`, which is eventually encoded into `[0x0, 0x68656c6c6f, 0x5]`.

## Scheme omission

Normally, the `scheme:` prefix is required for opting in to argument resolution. However, there are a few exceptions:

- the `addr:` prefix can be omitted when an address is expected;
- the `selector:` prefix can be omitted when a selector is expected;
- the `storage:` prefix can be omitted in the `starkli storage` command.

As an example, consider the `starkli invoke` command. To use the `addr` and `selector` schemes, one would run:

```console
starkli invoke addr:eth selector:transfer ...
```

However, since the first positional argument for the `starkli invoke` is always expected to be an address, and the second one a selector, this command can be simplified into:

```console
starkli invoke eth transfer ...
```
