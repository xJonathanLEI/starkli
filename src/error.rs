use std::fmt::Display;

use starknet::{
    accounts::{AccountError, AccountFactoryError},
    core::types::StarknetError,
    providers::ProviderError,
};

/// Makes error details visible, as they're not displayed by default.
pub fn account_error_mapper<S>(err: AccountError<S>) -> anyhow::Error
where
    S: Display,
{
    match err {
        AccountError::Provider(ProviderError::StarknetError(err)) => map_starknet_error(err),
        err => anyhow::anyhow!("{}", err),
    }
}

/// Makes error details visible, as they're not displayed by default.
pub fn account_factory_error_mapper<S>(err: AccountFactoryError<S>) -> anyhow::Error
where
    S: Display,
{
    match err {
        AccountFactoryError::Provider(ProviderError::StarknetError(err)) => map_starknet_error(err),
        err => anyhow::anyhow!("{}", err),
    }
}

fn map_starknet_error(err: StarknetError) -> anyhow::Error {
    match err {
        StarknetError::ContractError(err) => {
            anyhow::anyhow!("ContractError: {}", err.revert_error.trim())
        }
        StarknetError::TransactionExecutionError(err) => {
            anyhow::anyhow!(
                "TransactionExecutionError (tx index {}): {}",
                err.transaction_index,
                err.execution_error.trim()
            )
        }
        StarknetError::ValidationFailure(err) => {
            anyhow::anyhow!("ValidationFailure: {}", err.trim())
        }
        StarknetError::UnexpectedError(err) => {
            anyhow::anyhow!("UnexpectedError: {}", err.trim())
        }
        err => anyhow::anyhow!("{}", err),
    }
}
