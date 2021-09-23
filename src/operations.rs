use crate::errors::AppError;
use std::str::FromStr;

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

pub enum OperType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl FromStr for OperType {
    type Err = AppError;

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
