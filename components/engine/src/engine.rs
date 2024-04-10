use std::io::Write;

use crate::{EngineError, Record};

pub(crate) mod client;
pub mod transaction;

pub(crate) use client::ClientId;
pub(crate) use transaction::TxId;

use crate::engine::{
    client::{Client, Clients},
    transaction::TransactionInfo,
};

pub(crate) struct Engine {
    clients: Clients,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self { clients: Default::default() }
    }

    pub(crate) async fn process_record(&mut self, r: Record) -> Result<(), EngineError> {
        log::info!("{:?}", &r);

        let client_id = r.client_id;
        let tx_info = TransactionInfo::from_record(r)?;

        if !self.clients.contains_key(&client_id) {
            self.clients.insert(client_id, Client::new());
        }

        let client =
            self.clients.get_mut(&client_id).ok_or(EngineError::ClientNotExists(client_id))?;
        client.process_transaction(tx_info).await?;

        Ok(())
    }

    pub(crate) async fn print_wallets<W: Write>(&mut self, mut out: W) -> Result<(), EngineError> {
        writeln!(out, "client,available,held,total")?;
        for (id, client) in self.clients.iter_mut() {
            let wallet = client.wallet().await?;
            writeln!(out, "{id},{wallet}")?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) async fn print_sorted_wallets<W: Write>(
        &mut self,
        mut out: W,
    ) -> Result<(), EngineError> {
        use std::collections::BTreeSet;

        write!(out, "client,available,held,total")?;

        //i need to sort these in case of testing
        let mut sorted_clients: BTreeSet<String> = BTreeSet::new();
        for (id, client) in self.clients.iter_mut() {
            let wallet = client.wallet().await?;
            sorted_clients.insert(format!("\n{id},{wallet}"));
        }

        for str in sorted_clients.iter() {
            write!(out, "{}", str)?;
        }
        Ok(())
    }
}
