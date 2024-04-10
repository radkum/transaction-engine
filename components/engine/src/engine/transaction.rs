use std::collections::BTreeMap;

use crate::{EngineError, Record};

pub(crate) type TxId = u16;
pub(super) type Transactions = BTreeMap<TxId, TxResult>;

#[derive(Debug)]
pub(crate) enum TxAction {
    Deposit(f32),
    Withdrawal(f32),
    Dispute,
    Resolve,
    Chargeback,
    Close,
}

#[derive(Debug)]
pub(crate) enum TxResult {
    Deposited(f32),
    Disputed(f32),
}

#[derive(Debug)]
pub(crate) struct TransactionInfo {
    id: TxId,
    tx: TxAction,
}

impl TransactionInfo {
    pub(crate) fn close() -> Self {
        Self { id: u16::MAX, tx: TxAction::Close }
    }

    pub(crate) fn from_record(r: Record) -> Result<Self, EngineError> {
        let tx = match r.ty.as_str() {
            "deposit" => {
                let Some(amount) = r.amount else {
                    return Err(EngineError::RecordError(
                        "The amount field is missing for deposit transaction in csv".to_string(),
                    ));
                };
                TxAction::Deposit(amount)
            },
            "withdrawal" => {
                let Some(amount) = r.amount else {
                    return Err(EngineError::RecordError(
                        "The amount field is missing for withdraw transaction in csvmissing \
                         amount field for withdrawal in csv"
                            .to_string(),
                    ));
                };
                TxAction::Withdrawal(amount)
            },
            "dispute" => {
                let None = r.amount else {
                    return Err(EngineError::RecordError(
                        "The amount should be empty for dispute in csv".to_string(),
                    ));
                };
                TxAction::Dispute
            },
            "resolve" => {
                let None = r.amount else {
                    return Err(EngineError::RecordError(
                        "The amount should be empty for resolve in csv".to_string(),
                    ));
                };
                TxAction::Resolve
            },
            "chargeback" => {
                let None = r.amount else {
                    return Err(EngineError::RecordError(
                        "The amount should be empty for chargeback in csv".to_string(),
                    ));
                };
                TxAction::Chargeback
            },
            _ => return Err(EngineError::RecordError("Unknown transaction type".to_string())),
        };

        Ok(Self { id: r.tx_id, tx })
    }

    pub(crate) fn tx(&self) -> &TxAction {
        &self.tx
    }

    pub(crate) fn id(&self) -> TxId {
        self.id
    }
}
