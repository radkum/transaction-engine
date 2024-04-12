use std::collections::HashMap;

use tokio::{
    sync::{
        mpsc,
        mpsc::{error::SendError, Receiver, Sender},
    },
    task::JoinHandle,
};
use wallet::Wallet;

use crate::{
    engine::transaction::{TransactionInfo, Transactions, TxAction, TxResult},
    EngineError,
};

mod wallet;

pub(crate) type ClientId = u32;
// unordered map is the best option. We don't need have it sorted
pub(super) type Clients = HashMap<ClientId, Client>;

pub(super) struct Client {
    sender: Sender<TransactionInfo>,
    join_handle: Option<JoinHandle<Wallet>>,
}

impl Client {
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::channel::<TransactionInfo>(32);

        let mut client = Self { sender: tx, join_handle: None };

        // we want process transactions for each client separately. It allows as to parallelize work
        // and get better performance. There are disadvantages (eg. each tokio task must work till to
        // finish of program, regardless if do something or not) but it can be easy mitigated (eg.
        // add timeout to receiver and then save a state, and start only when new transaction is sent)
        client.run(rx);
        client
    }

    fn run(&mut self, mut receiver: Receiver<TransactionInfo>) {
        let handle = tokio::spawn(async move {
            let mut wallet = Wallet::default();
            let mut tx_history = Transactions::default();

            // messages are received till to TxAction::Close message. Then task return wallet
            while let Some(tx_info) = receiver.recv().await {
                if wallet.locked() {
                    // only close action works. Other should be skipped till to unlocking client
                    if let TxAction::Close = tx_info.tx() {
                        receiver.close();
                    }
                    continue;
                }

                match tx_info.tx() {
                    TxAction::Deposit(amount) => {
                        wallet.deposit(*amount);
                        tx_history.insert(tx_info.id(), TxResult::Deposited(*amount));
                    },
                    TxAction::Withdrawal(amount) => {
                        wallet.withdrawal(*amount);
                    },
                    TxAction::Dispute => {
                        let Some(deposit_tx) = tx_history.get(&tx_info.id()) else {
                            log::warn!("There is no saved transaction with id: {}", tx_info.id());
                            continue;
                        };

                        let TxResult::Deposited(amount) = deposit_tx else {
                            log::warn!("There is no deposited transaction to dispute");
                            continue;
                        };

                        if wallet.dispute(*amount) {
                            // instead of deleting the deposited transaction, we simply insert a new
                            // disputed transaction that replaces the old one
                            tx_history.insert(tx_info.id(), TxResult::Disputed(*amount));
                        }
                    },
                    TxAction::Resolve => {
                        let Some(disputed_tx) = tx_history.get(&tx_info.id()) else {
                            log::warn!("There is no saved transaction with id: {}", tx_info.id());
                            continue;
                        };

                        let TxResult::Disputed(amount) = disputed_tx else {
                            log::warn!("There is no disputed transaction to resolve");
                            continue;
                        };

                        wallet.resolve(*amount);

                        // instead of deleting the disputed transaction, we simply insert a new
                        // deposited transaction that replaces the old one
                        tx_history.insert(tx_info.id(), TxResult::Deposited(*amount));
                    },
                    TxAction::Chargeback => {
                        let Some(disputed_tx) = tx_history.get(&tx_info.id()) else {
                            log::warn!("There is no saved transaction with id: {}", tx_info.id());
                            continue;
                        };

                        let TxResult::Disputed(amount) = disputed_tx else {
                            log::warn!("There is no disputed transaction to chargeback");
                            continue;
                        };

                        wallet.chargeback(*amount);

                        // we can't simply replace a dispute transaction with another in this case,
                        // because charge back revert an transaction. Therefore we remove it from history
                        tx_history.remove(&tx_info.id());
                    },
                    TxAction::Close => receiver.close(),
                }
            }

            // Once all operations are completed, return the wallet
            // that represent the client's transaction status
            wallet
        });

        self.join_handle = Some(handle)
    }

    pub(super) async fn process_transaction(
        &mut self,
        tx_info: TransactionInfo,
    ) -> Result<(), SendError<TransactionInfo>> {
        self.sender.send(tx_info).await?;
        Ok(())
    }

    pub(super) async fn wallet(&mut self) -> Result<Wallet, EngineError> {
        self.close().await?;
        if let Some(wallet) = &mut self.join_handle {
            Ok(wallet.await?)
        } else {
            // this should not happen. unreachable! or error?
            Err(EngineError::FailedToGetWallet)
        }
    }

    pub(super) async fn close(&mut self) -> Result<(), SendError<TransactionInfo>> {
        // the easiest solution to close wallet computation is to send proper message
        self.process_transaction(TransactionInfo::close()).await?;
        Ok(())
    }
}
