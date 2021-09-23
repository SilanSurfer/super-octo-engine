use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Couldn't convert {0} to enum")]
    FromStrError(String),
    #[error("Not enough funds ({1}) in the account to withdraw ({2}), tx = {0}")]
    NotEnoughFundsToWithdraw(u32, f32, f32),
    #[error("Not enough funds ({1}) in the account to held ({2}), tx = {0}")]
    NotEnoughFundsToHeld(u32, f32, f32),
    #[error("Client ID = {1} doesn't exist, tx = {0}")]
    ClientDoesnExist(u32, u16),
    #[error("Referenced transaction ({0}) doesn't exist")]
    ReferencedTransDoesntExist(u32),
    #[error("Referenced transaction ({0}) isn't uder dispute")]
    ReferencedTransIsNotDisputed(u32),
    #[error("Account of client ({1}) is locked, tx = {0}")]
    AccountIsLocked(u32, u16),
}
