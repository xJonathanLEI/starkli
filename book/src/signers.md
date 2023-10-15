# Signers

Starkli uses "signers" to sign transactions. Technically speaking, a signer can be anything that can provide valid signatures for transactions. In practice, the following signer types are currently supported:

- [encrypted keystores](#encrypted-keystores)
- [plain text private keys](#plain-text-private-keys)

More signer types will be supported as they become available. As of this writing, the most secure signer type is encrypted keystores.

Signers can be created and managed through the `starkli signer` command.

## Encrypted keystores

Encrypted keystores are JSON files that follow the [Web3 secret storage definition](https://ethereum.org/en/developers/docs/data-structures-and-encoding/web3-secret-storage/). A password must be supplied to create a keystore, and is required for subsequently using the keystore.

> ⚠️ **Warning**
>
> Keystores are encrypted, but they're only as secure as the password you used to create them.

To create a fresh keystore from scratch:

```console
starkli signer keystore new /path/to/keystore
```

and a keystore file will be created at `/path/to/keystore`.

You can then use it via the `--keystore <PATH>` option for commands expecting a signer.

Alternatively, you can set the `STARKNET_KEYSTORE` environment variable to make command invocations easier:

```console
export STARKNET_KEYSTORE="/path/to/keystore"
```

> ℹ️ **Note**
>
> Even when `STARKNET_KEYSTORE` is set, it would be ignored by Starkli when any other signer option is supplied via the command line, including using the `--keystore <PATH>` option.

## Plain text private keys

> ⚠️ **Warning**
>
> Using plain text private keys is highly insecure. Never use this for production.

Plain text private keys are only intended to be used for development purposes, where security of keys does not matter. To generate a private key, run the command:

```console
starkli signer gen-keypair
```

For commands that expect a signer, you can then use the `--private-key <KEY>` option. Alternatively, you can set the `STARKNET_PRIVATE_KEY` environment variable to make command invocations easier.

> ℹ️ **Note**
>
> Starkli shows a warning when you use plain-text private keys. If you know what you're doing, you can suppress this warning by setting the `STARKLI_NO_PLAIN_KEY_WARNING` to _anything_ but `false`.
