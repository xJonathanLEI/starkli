# Accounts

Starkli sends out transactions through accounts. Starknet natively supports [account abstraction](https://ethereum.org/en/roadmap/account-abstraction/) and all accounts are smart contracts. Therefore, there are many "flavors" of accounts and Starkli supports the most popular ones. Starkli refers to these "flavors" as _variants_.

Currently, the only supported variant is [OpenZeppelin's account contract implementation](https://github.com/OpenZeppelin/cairo-contracts/blob/70cbd05ed24ccd147f24b18c638dbd6e7fea88bb/src/openzeppelin/account/presets/Account.cairo), but many more are expected to be supported soon.

Accounts can be created and managed through the `starkli account` command. Variant-specific commands are available under `starkli account <VARIANT>`.

## Account creation

Before creating an account, you must first decide on the _variant_ to use. As of this writing, the only supported variant is `oz`, the OpenZeppelin account contract.

All variants come with an `init` subcommand that creates an account file ready to be deployed. For example, to create an `oz` account:

```console
starkli account oz init /path/to/account
```

> ℹ️ **Note**
>
> The `starkli account oz init <PATH>` command requires a signer. Starkli would complain that a signer is missing when running the command as shown, unless a keystore is specified via the `STARKNET_KEYSTORE` environment variable. See the [signers page](./signers.md) page for more details.

## Account deployment

Once you have an account file, you can deploy the account contract with the `starkli account deploy` command. This command sends a `DEPLOY_ACCOUNT` transaction, which requires the account to be funded with some `ETH` for paying for the transaction fee.

For example, to deploy the account we just created:

```console
starkli account deploy /path/to/account
```

> ℹ️ **Note**
>
> This command also requires a signer. You must provide the same signer used for creating the account file in the first place.

When run, the command shows the address where the contract will be deployed on, and instructs the user to fund the account before proceeding. Here's an example command output:

```console
The estimated account deployment fee is 0.000011483579723913 ETH. However, to avoid failure, fund at least:
    0.000017225369585869 ETH
to the following address:
    0x01cf4d57ba01109f018dec3ea079a38fc08b789e03de4df937ddb9e8a0ff853a
Press [ENTER] once you've funded the address.
```

Once the account deployment transaction is confirmed, the account file will be update to reflect the deployment status. It can then be used for commands where an account is expected.

## Account fetching

Account fetching allows recreating the account file from on-chain data alone. This could be helpful when:

- the account file is lost; or
- migrating an account from another tool/application.

The `starkli account fetch` commands creates an account file using just the address provided:

```console
starkli account fetch <ADDRESS> --output /path/to/account
```

Running the command above creates the account file at `/path/to/account`.
