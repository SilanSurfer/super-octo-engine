use crate::data_records::InputRecord;
use crate::errors::TransactionError;
use crate::operations::{OperType, Operation};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Clone)]
struct Account {
    available: f32,
    held: f32,
    locked: bool,
    disputed_trans: u32,
    operations: HashMap<u32, Operation>,
}

pub struct Engine {
    database: Mutex<HashMap<u16, Account>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            database: Mutex::new(HashMap::new()),
        }
    }

    pub fn output(&mut self) {
        // Had to clone database because you can't iterate over MutexGuard...
        // Check if there is better way to do this...
        let database = self.database.lock().expect("Lock poisoned!").clone();
        println!("client, available, held, total, locked");
        for (client_id, account) in database {
            let total = account.available + account.held;
            println!(
                "{}, {:.4}, {:.4}, {:.4}, {}",
                client_id, account.available, account.held, total, account.locked
            );
        }
    }

    pub async fn process_record(&mut self, record: InputRecord) -> Result<(), TransactionError> {
        let operation = OperType::from_str(&record.oper_type)?;
        match operation {
            OperType::Deposit => self.handle_deposit(&record).await,
            OperType::Withdrawal => self.handle_withdrawal(&record).await,
            OperType::Dispute => self.handle_dispute(&record).await,
            OperType::Resolve => self.handle_resolve(&record).await,
            OperType::Chargeback => self.handle_chargeback(&record).await,
        }
    }

    // TODO: Refactor all "handle" functions...
    async fn handle_deposit(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        let mut database = self.database.lock().expect("Lock poisoned!");

        match database.get_mut(&record.client) {
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
                database.insert(
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

    async fn handle_withdrawal(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        let mut database = self.database.lock().expect("Lock poisoned!");

        match database.get_mut(&record.client) {
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

    async fn handle_dispute(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        let mut database = self.database.lock().expect("Lock poisoned!");

        match database.get_mut(&record.client) {
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

    async fn handle_resolve(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        let mut database = self.database.lock().expect("Lock poisoned!");

        match database.get_mut(&record.client) {
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

    async fn handle_chargeback(&mut self, record: &InputRecord) -> Result<(), TransactionError> {
        let mut database = self.database.lock().expect("Lock poisoned!");

        match database.get_mut(&record.client) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Write macro for assertions to simplify code
    #[tokio::test]
    async fn deposit_when_client_does_not_exist() {
        let mut engine = Engine::new();
        let client_id: u16 = 1;
        let tx: u32 = 123;
        let input_record = InputRecord {
            oper_type: "deposit".to_string(),
            client: client_id,
            tx,
            amount: 34.5,
        };

        let mut expected_operations = HashMap::new();
        expected_operations.insert(tx, Operation::new(OperType::Deposit, 34.5));
        let expected_account_state = Account {
            available: 34.5,
            held: 0.0,
            locked: false,
            disputed_trans: 0,
            operations: expected_operations,
        };

        assert!(engine.process_record(input_record).await.is_ok());

        let database = engine.database.lock().unwrap();
        let actual_account_state = database.get(&client_id);
        assert!(actual_account_state.is_some());

        let actual_account_state = actual_account_state.unwrap();

        assert_eq!(
            expected_account_state.available,
            actual_account_state.available
        );
        assert_eq!(expected_account_state.held, actual_account_state.held);
        assert_eq!(expected_account_state.locked, actual_account_state.locked);
        assert_eq!(
            expected_account_state.disputed_trans,
            actual_account_state.disputed_trans
        );
        assert_eq!(
            expected_account_state.operations.get(&tx).unwrap(),
            actual_account_state.operations.get(&tx).unwrap()
        );
    }

    #[tokio::test]
    async fn withdraw_money_from_account() {
        let mut engine = Engine::new();
        let client_id: u16 = 1;
        let tx: u32 = 123;
        let deposit_record = InputRecord {
            oper_type: "deposit".to_string(),
            client: client_id,
            tx: 12,
            amount: 34.5,
        };

        assert!(engine.process_record(deposit_record).await.is_ok());

        let input_record = InputRecord {
            oper_type: "withdrawal".to_string(),
            client: client_id,
            tx,
            amount: 14.5,
        };

        let mut expected_operations = HashMap::new();
        expected_operations.insert(12, Operation::new(OperType::Deposit, 34.5));
        expected_operations.insert(tx, Operation::new(OperType::Withdrawal, 14.5));
        let expected_account_state = Account {
            available: 20.0,
            held: 0.0,
            locked: false,
            disputed_trans: 0,
            operations: expected_operations,
        };

        assert!(engine.process_record(input_record).await.is_ok());

        let database = engine.database.lock().unwrap();
        let actual_account_state = database.get(&client_id);
        assert!(actual_account_state.is_some());

        let actual_account_state = actual_account_state.unwrap();

        assert_eq!(
            expected_account_state.available,
            actual_account_state.available
        );
        assert_eq!(expected_account_state.held, actual_account_state.held);
        assert_eq!(expected_account_state.locked, actual_account_state.locked);
        assert_eq!(
            expected_account_state.disputed_trans,
            actual_account_state.disputed_trans
        );
        assert_eq!(actual_account_state.operations.len(), 2 as usize);
        assert_eq!(
            expected_account_state.operations.get(&tx).unwrap(),
            actual_account_state.operations.get(&tx).unwrap()
        );
    }
}
