use crate::data_records::InputRecord;
use crate::errors::TransactionError;
use crate::operations::{OperType, Operation};

use std::collections::HashMap;
use std::str::FromStr;

struct Account {
    available: f32,
    held: f32,
    locked: bool,
    disputed_trans: u32,
    operations: HashMap<u32, Operation>,
}

pub struct Engine {
    database: HashMap<u16, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            database: HashMap::new(),
        }
    }

    pub fn process_record(&mut self, record: InputRecord) -> Result<(), TransactionError> {
        let operation = OperType::from_str(&record.oper_type)?;
        match operation {
            OperType::Deposit => self.handle_deposit(&record),
            OperType::Withdrawal => self.handle_withdrawal(&record),
            OperType::Dispute => self.handle_dispute(&record),
            OperType::Resolve => self.handle_resolve(&record),
            OperType::Chargeback => self.handle_chargeback(&record),
        }
    }

    fn handle_deposit(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        match self.database.get_mut(&record.client) {
            Some(account) => {
                if !account.locked {
                    account.available += record.amount;
                    account
                        .operations
                        .insert(record.tx, Operation::new(OperType::Deposit, record.amount));
                    Ok(())
                } else {
                    Err(TransactionError::AccountIsLocked(record.tx, record.client))
                }
            }
            None => {
                let mut operations = HashMap::new();
                operations.insert(record.tx, Operation::new(OperType::Deposit, record.amount));
                self.database.insert(
                    record.client,
                    Account {
                        available: record.amount,
                        held: 0.0,
                        locked: false,
                        disputed_trans: 0,
                        operations,
                    },
                );
                Ok(())
            }
        }
    }

    fn handle_withdrawal(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        match self.database.get_mut(&record.client) {
            Some(account) => {
                if !account.locked {
                    if account.available >= record.amount {
                        account.available -= record.amount;
                        account.operations.insert(
                            record.tx,
                            Operation::new(OperType::Withdrawal, record.amount),
                        );
                        Ok(())
                    } else {
                        Err(TransactionError::NotEnoughFundsToWithdraw(
                            record.tx,
                            account.available,
                            record.amount,
                        ))
                    }
                } else {
                    Err(TransactionError::AccountIsLocked(record.tx, record.client))
                }
            }
            None => Err(TransactionError::ClientDoesnExist(record.tx, record.client)),
        }
    }

    fn handle_dispute(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        match self.database.get_mut(&record.client) {
            Some(account) => match account.operations.get_mut(&record.tx) {
                Some(transaction) => {
                    if account.available >= transaction.amount {
                        account.available -= transaction.amount;
                        account.held += transaction.amount;
                        transaction.under_dispute = true;
                        account.locked = true;
                        account.disputed_trans += 1;
                        Ok(())
                    } else {
                        Err(TransactionError::NotEnoughFundsToHeld(
                            record.tx,
                            account.available,
                            transaction.amount,
                        ))
                    }
                }
                None => Err(TransactionError::ReferencedTransDoesntExist(record.tx)),
            },
            None => Err(TransactionError::ClientDoesnExist(record.tx, record.client)),
        }
    }

    fn handle_resolve(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        match self.database.get_mut(&record.client) {
            Some(account) => {
                match account.operations.get_mut(&record.tx) {
                    Some(transaction) => {
                        if transaction.under_dispute {
                            // In theory it's possible that there is not enough held funds,
                            // but it means that there is error elsewhere and it shouldn't happen.
                            account.held -= transaction.amount;
                            account.available += transaction.amount;
                            transaction.under_dispute = false;
                            account.disputed_trans -= 1;
                            if account.disputed_trans == 0 {
                                account.locked = false;
                            }
                            Ok(())
                        } else {
                            Err(TransactionError::ReferencedTransIsNotDisputed(record.tx))
                        }
                    }
                    None => Err(TransactionError::ReferencedTransDoesntExist(record.tx)),
                }
            }
            None => Err(TransactionError::ClientDoesnExist(record.tx, record.client)),
        }
    }

    fn handle_chargeback(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        match self.database.get_mut(&record.client) {
            Some(account) => {
                match account.operations.get_mut(&record.tx) {
                    Some(transaction) => {
                        if transaction.under_dispute {
                            // In theory it's possible that there is not enough held funds,
                            // but it means that there is error elsewhere and it shouldn't happen.
                            account.held -= transaction.amount;
                            transaction.under_dispute = false;
                            account.disputed_trans -= 1;
                            if account.disputed_trans == 0 {
                                account.locked = false;
                            }
                            Ok(())
                        } else {
                            Err(TransactionError::ReferencedTransIsNotDisputed(record.tx))
                        }
                    }
                    None => Err(TransactionError::ReferencedTransDoesntExist(record.tx)),
                }
            }
            None => Err(TransactionError::ClientDoesnExist(record.tx, record.client)),
        }
    }
}
