# Declaring classes

In Starknet, all deployed contracts are instances of certain declared classes. Therefore, the first step of deploying a contract is declaring a class, if it hasn't been declared already.

With Starkli, this is done with the `starkli declare` command.

> ℹ️ **Note**
>
> You need both a [signer](./signers.md) and an [account](./accounts.md) for this. The commands shown in this page omit the signer and account options for better readability, and assume you've properly configured the environment variables.

You can declare Sierra classes, which are the output of the `starknet-compile` command, or `target/dev/xxxx.contract_class.json` files if you're using `Scarb`.

> ℹ️ **Note**
>
> Starting from Starkli v0.4.0, declaring Cairo 0 (legacy) classes is no longer supported. To declare such a class, use a v0.3.x Starkli release instead.

To declare any class, simply run:

```console
starkli declare /path/to/class/file
```

Once the declaration is successful, Starkli displays the class hash declared. The class hash is needed for [deploying contracts](./deploying-contracts.md).

## Sierra class compilation

When declaring Sierra classes, Starknet requires a so-called _CASM hash_ to be provided. This is important because as of this writing, the Sierra-to-CASM compilation process isn't proven by the OS. Should the _CASM hash_ not be provided and signed by the user, a malicious sequencer would be able to claim anything to be the CASM output, effectively deploying arbitrary code.

To come up with the _CASM hash_, Starkli compiles the Sierra class provided under the hood. Starkli ships with different versions of Sierra compilers built-in so that declaration does not depend on Cairo toolchain installation.

> ℹ️ **Note**
>
> If you encounter an error similar to `unsupported Sierra version: x.y.z`, try upgrading Starkli, or file a bug if you're already on the latest release.

> ℹ️ **Note**
>
> For advanced users, it's possible to skip the Sierra-to-CASM compilation process by directly providing a `--casm-hash <CASM_HASH>`.

## Redeclaring classes

While the normal process of declaring a class involves getting the compiled contract artifact from the compiler and following the steps documented above, it's sometimes helpful to _redeclare_ a class you found from another network.

To do so, simply run `starkli declare` on any class fetched from `class-at` or `class-by-hash` commands. For example:

```console
starkli class-by-hash --network sepolia SOME_CLASS_HASH_HERE > class.json
starkli declare --network mainnet ./class.json
```

> 💡 **Tips**
>
> While Starkli is capable of handling its declaration, the class format retrieved from JSON-RPC is different from the original compiler output. To obtain the format identical to what comes out of the compiler, use the `--parse` flag when retrieving the class (e.g. in a `class-by-hash` command).
>
> Note that while `--parse` should work just fine most of the time, unfortunately, certain exotic classes might not be parsable. Nevertheless, Starkli would still be able to redeclare the format without parsing.
