# Declaring classes

In Starknet, all deployed contracts are instances of certain declared classes. Therefore, the first step of deploying a contract is declaring a class, if it hasn't been declared already.

With Starkli, this is done with the `starkli declare` command.

> ℹ️ **Note**
>
> You need both a [signer](./signers.md) and an [account](./accounts.md) for this. The commands shown in this page omit the signer and account options for better readability, and assume you've properly configured the environment variables.

You can declare the following types of contract artifacts:

- Sierra classes: output of the `starknet-compile` command; and
- _(Deprecated)_ Legacy Cairo 0 classes: output of the `starknet-compile-deprecated` command

To declare any class, simply run:

```console
starkli declare /path/to/class/file
```

Starkli is capable of determining the type of class provided. There are no separate commands for Sierra and legacy classes.

Once the declaration is successful, Starkli displays the class hash declared. The class hash is needed for [deploying contracts](./deploying-contracts.md).

## Sierra class compilation

When declaring Sierra classes, Starknet requires a so-called _CASM hash_ to be provided. This is important because as of this writing, the Sierra-to-CASM compilation process isn't proven by the OS. Should the _CASM hash_ not be provided and signed by the user, a malicious sequencer would be able to claim anything to be the CASM output, effectively deploying arbitrary code.

To come up with the _CASM hash_, Starkli compiles the Sierra class provided under the hood. By default, it automatically chooses one of the compiler versions shipped with Starkli itself based on the network. Users can override the compiler version used by providing a `--compiler-version <VERSION>` option.

> ℹ️ **Note**
>
> Unless you're working with custom networks where it's infeasible for Starkli to detect the right compiler version, you shouldn't need to manually choose a version with `--compiler-version`.
>
> If Starkli _does_ choose the wrong compiler version, try upgrading Starkli, or file a bug if you're already on the latest release.

> ℹ️ **Note**
>
> For advanced users, it's possible to skip the Sierra-to-CASM compilation process by directly providing a `--casm-hash <CASM_HASH>`.
