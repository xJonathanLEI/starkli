# Profiles

Starkli supports profiles, where custom networks can be defined.

> ℹ️ **Note**
>
> Profiles only allow defining custom networks at the moment. More features will be added soon.

> ℹ️ **Note**
>
> Currently, only one profile `default` is supported. Defining additional profiles results in an error.

## The profiles file

Depending on what operating system you're using, the profiles file are located at:

- Linux and Mac: `~/.config/starkli/profiles.toml`
- Windows: `%AppData%\starkli\profiles.toml`

> 💡 **Tips**
>
> The profiles file is created automatically the first time you use a [free RPC vendor](./providers.md#free-rpc-vendors). You can take the automatically generated file as a starting point for adding new networks.

## Defining custom networks

Custom networks can be defined as `<PROFILE_ID>.networks.<NETWORK_ID>`. Since only the `default` profile is supported at the moment, networks should be defined as `default.networks.<NETWORK_ID>`.

Each network contains the following properties:

| Field      | Mandatory | Type              | Description                                       |
| ---------- | --------- | ----------------- | ------------------------------------------------- |
| `name`     | No        | `String`          | Human-readable network name, currently unused     |
| `chain_id` | Yes       | `String`          | String representation of the chain ID             |
| `provider` | Yes       | `String`/`Object` | [Provider configuration](#provider-configuration) |

### Provider configuration

The `provider` field can be either a `String` or an `Object`. When the `provider` value is an `Object`, it must contain a `type` field, whose value must be one of the following:

| Value                            | Description                                              |
| -------------------------------- | -------------------------------------------------------- |
| [`rpc`](#rpc-provider-variant)   | Use the JSON-RPC provider by specifying an endpoint URL  |
| [`free`](#free-provider-variant) | Use a [free RPC vendor](./providers.md#free-rpc-vendors) |

#### `rpc` provider variant

| Field  | Mandatory | Type     | Description                  |
| ------ | --------- | -------- | ---------------------------- |
| `type` | Yes       | `String` | Value must be `rpc`          |
| `url`  | Yes       | `String` | URL to the JSON-RPC endpoint |

#### `free` provider variant

| Field    | Mandatory | Type     | Description                             |
| -------- | --------- | -------- | --------------------------------------- |
| `type`   | Yes       | `String` | Value must be `free`                    |
| `vendor` | Yes       | `String` | Must be one of `blast` and `nethermind` |

#### `rpc` provider shorthand

The `provider` value can also be a `String`. When this is the case, it's used as a shorthand for the [`rpc` variant](#rpc-provider-variant). So this value:

```toml
provider = "https://example.com/"
```

is the same as this:

```toml
provider = { type = "rpc", url = "https://example.com/" }
```

### Example network configurations

This section contains a few example network configurations.

#### Network with the RPC provider

```toml
[default.networks.mainnet]
chain_id = "SN_MAIN"
provider = { type = "rpc", url = "https://example.com/" }
```

#### Network with the RPC provider shorthand

```toml
[default.networks.mainnet]
chain_id = "SN_MAIN"
provider = "https://example.com/"
```

#### Network with the free RPC vendor

```toml
[default.networks.mainnet]
chain_id = "SN_MAIN"
provider = { type = "free", vendor = "blast" }
```
