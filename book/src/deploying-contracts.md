# Deploying contracts

Once you obtain a class hash by [declaring a class](./declaring-classes.md), it's possible to deploy instances of the class.

With Starkli, this is done with the `starkli deploy` command.

> ℹ️ **Note**
>
> You need both a [signer](./signers.md) and an [account](./accounts.md) for this. The commands shown in this page omit the signer and account options for better readability, and assume you've properly configured the environment variables.

To deploy a contract with class hash `<CLAS_HASH>`, simply run:

```console
starkli deploy <CLASS_HASH> <CTOR_ARGS>
```

where `<CTOR_ARGS>` is the list of constructor arguments, if any.

> 💡 **Tips**
>
> You might be able to leverage [argument resolution](./argument-resolution.md) to simplify the argument list input.

Under the hood, Starkli sends an `INVOKE` transaction to the [Universal Deployer Contract](https://community.starknet.io/t/universal-deployer-contract-proposal/), as Starknet does not support native external contract deployment transactions.
