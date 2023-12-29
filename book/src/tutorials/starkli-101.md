# Starkli 101

In this tutorial, you will be guided from scratch to deploy contracts on the Starknet testnet.

1. Preparing your cryptographic keys and account.
2. Compiling your Cairo contract with a simple example contract.
3. Declaring a new class on Starknet.
4. Deploying a contract instance of a class on Starknet.

> ℹ️ **Note**
>
> To make it easier to get started, this tutorial skips the step of choosing a provider and uses the `goerli` network fallback with [free RPC vendor](../providers.md#free-rpc-vendors), but relying on the network fallback is discouraged in practice.
>
> Make sure to visit the [providers](../providers.md) page to learn more once you finish the tutorial.

## Prepare a signer and an account

To interact with the network, you need an account to sign transactions. This tutorial assumes that you have no signer or account setup.

### Initialize a signer

The signer is in charge of signing transactions. You can create a new pair of cryptographic key, or import an existing private key. Check out the [signers](../signers.md) page for more details.

Let's create a new signer here.

```console
$ starkli signer keystore new /path/to/key.json
```

This will prompt you to enter a password and save your encrypted private key into the `key.json` [keystore](../signers.md#encrypted-keystores) file.

To then use this keystore at each command, you can export the environment variable like:

```console
$ export STARKNET_KEYSTORE="/path/to/key.json"
```

### Initialize an account

We then initialize a new account, using OpenZeppelin class already declared on Starknet:

```console
$ starkli account oz init /path/to/account.json
```

> ℹ️ **Note**
>
> Note that we didn't need to pass the `--keystore` option since the `STARKNET_KEYSTORE` environment variable is already set.

### Fund and deploy the account

Up to this point, only the parameters for deploying the account have been generated, but the account itself hasn't been deployed yet. You must deploy the account to be able to use it.

To deploy the account, run:

```console
$ starkli account deploy /path/to/account.json
```

The command would then displays a message similar to this:

```console
The estimated account deployment fee is 0.000004323000964029 ETH. However, to avoid failure, fund at least:
    0.000006484501446043 ETH
to the following address:
    0x077c********************************************************3f8a
Press [ENTER] once you've funded the address.
```

> ℹ️ **Note**
>
> The address above is redacted just in case you accidentally send funds to it.

As instructed, you must pre-fund the address with `ETH` to be able to continue. Send enough `ETH` to the address, and then press the Enter key.

> 💡 **Tips**
>
> You can use the `starkli balance` command (probably in a separate terminal session) to check whether the destinated account address has been successfully funded.

An account deployment transaction will be sent out. Once the transaction is confirmed, your account will be ready to use.

Again, to avoid having to pass the account file to each command invocation, we can export the `STARKNET_ACCOUNT` variable:

```console
$ export STARKNET_ACCOUNT="/path/to/account.json"
```

For more details about accounts, please refer to the [accounts](../accounts.md) page.

## Compile your Cairo contract

The next step is to compile a Cairo contract. There are a few options for compiling Cairo contracts, but in this tutorial, we'll use [Scarb](https://docs.swmansion.com/scarb/docs).

> ℹ️ **Note**
>
> This tutorial uses Scarb `v0.6.1`.

First, create a Scrab project:

```console
$ mkdir my_contract
$ cd ./my_contract/
$ scarb init
```

Update the `Scarb.toml` file to include the `starknet` dependency and add the `starknet-contract` target:

```toml
[package]
name = "my_contract"
version = "0.1.0"

[dependencies]
starknet = "=2.1.0"

[[target.starknet-contract]]
```

Then, replace the `./src/lib.cairo` file with the following content:

```rust
// ** ./src/lib.cairo **

#[starknet::interface]
trait MyContractInterface<T> {
    fn name_get(self: @T) -> felt252;
    fn name_set(ref self: T, name: felt252);
}

#[starknet::contract]
mod my_contract {
    #[storage]
    struct Storage {
        name: felt252,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        NameChanged: NameChanged,
    }

    #[derive(Drop, starknet::Event)]
    struct NameChanged {
        previous: felt252,
        current: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, name: felt252) {
        self.name.write(name);
    }

    #[external(v0)]
    impl MyContract of super::MyContractInterface<ContractState> {
        fn name_get(self: @ContractState) -> felt252 {
            self.name.read()
        }

        fn name_set(ref self: ContractState, name: felt252) {
            let previous = self.name.read();
            self.name.write(name);
            self.emit(NameChanged { previous, current: name });
        }
    }
}
```

Now build the contract:

```console
$ scarb build
```

and the built contract artifact (aka. Sierra class) will be available at `./target/dev/my_contract_my_contract.sierra.json`.

## Declare the new class

Contract classes are like blueprints where actual contract instances can be deployed from. Now it's time to declare the new Sierra class. To declare, simply run:

```console
$ starkli declare --watch ./target/dev/my_contract_my_contract.sierra.json
```

> ℹ️ **Note**
>
> If the class hash already exists, Starkli will not send out a declaration transaction.
>
> This is _not_ considered an error, but if you want to try a declaration anyways, you can do so by making your class slightly different by changing the Cairo code.

Starkli will then output the Cairo 1 class hash (which can also be obtained using the `starkli class-hash <FILE.json>` command). You'll need this class hash for deploying the contract.

If you followed the exact same steps with the exact same tooling versions, you should be getting the class hash of `0x0756ea65987892072b836b9a56027230bbe8fbed5e0370cff22778d071a0798e`. It's normal that you arrive at a different hash, so don't worry about that.

## Deploy your contract

Once your new class is declared, you can deploy an instance of your contract. Concrete contract instances hold state (storage values) while classes (what you declared at the last step) define the logic.

With Starkli, this is done via the `deploy` command. To deploy, you will need the class hash obtained in the last step when declaring. Apart from the hash, you will also need to pass the constructor arguments to your contract:

```console
$ starkli deploy <CLASS_HASH> <CTOR_ARGS>
```

In the case of `my_contract` we declared above, the contract is expecting a single `felt252` to be used as the name. A common way to represent strings as `felt252` is to use the [Cairo short string](https://book.starknet.io/chapter_2/strings.html#working_with_short_strings) format. For example, you can find the Cairo short string representation of `"starkli"` with the `to-cairo-string` command:

```console
$ starkli to-cairo-string starkli
0x737461726b6c69
```

You can use this value directly as an argument to deploy the contract:

```console
$ starkli deploy --watch 0x0756ea65987892072b836b9a56027230bbe8fbed5e0370cff22778d071a0798e 0x737461726b6c69
```

> 💡 **Tips**
>
> You can leverage [argument resolution](../argument-resolution.md) to simplify the argument list:
>
> ```console
> $ starkli deploy --watch 0x0756ea65987892072b836b9a56027230bbe8fbed5e0370cff22778d071a0798e str:starkli
> ```
>
> Note how `0x737461726b6c69` is replaced with `str:starkli`. Learn more about argument resolution [here](../argument-resolution.md).

Starkli prints the deployed contract address in the command output. You can use the address for interacting with the deployed contract.

Here we will use the address `0x06d8e1f3ed72fc87aa896639a0f50a4b9e59adb24de8a42b477957e1a7996e1b`. You _will_ get a different address when you deploy the contract yourself. Simply replace the addresses in the following commands with your own.

Let's query the current name set for the contract:

```console
$ starkli call 0x06d8e1f3ed72fc87aa896639a0f50a4b9e59adb24de8a42b477957e1a7996e1b name_get
[
    "0x00000000000000000000000000000000000000000000000000737461726b6c69"
]
```

which is the Cairo short string representation of the text `"starkli"`.

Let's change it to `"starknet"` instead:

```console
$ starkli invoke --watch 0x06d8e1f3ed72fc87aa896639a0f50a4b9e59adb24de8a42b477957e1a7996e1b name_set str:starknet
```

Now query the name again:

```console
$ starkli call 0x06d8e1f3ed72fc87aa896639a0f50a4b9e59adb24de8a42b477957e1a7996e1b name_get
[
    "0x000000000000000000000000000000000000000000000000737461726b6e6574"
]
```

And the name returned has changed. We've successfully modified the state of our contract.
