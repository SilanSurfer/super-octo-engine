use crate::errors::TransactionError;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    pub trans_type: OperType,
    pub amount: f32,
    pub under_dispute: bool,
}

impl Operation {
    pub fn new(operation: OperType, amount: f32) -> Self {
        Self {
            trans_type: operation,
            amount,
            under_dispute: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl FromStr for OperType {
    type Err = TransactionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(OperType::Deposit),
            "withdrawal" => Ok(OperType::Withdrawal),
            "dispute" => Ok(OperType::Dispute),
            "resolve" => Ok(OperType::Resolve),
            "chargeback" => Ok(OperType::Chargeback),
            _ => Err(Self::Err::FromStrError(s.to_string())),
        }
    }
}
