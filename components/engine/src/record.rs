use crate::engine::{ClientId, TxId};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    pub(crate) ty: String,
    #[serde(rename = "client")]
    pub(crate) client_id: ClientId,
    #[serde(rename = "tx")]
    pub(crate) tx_id: TxId,
    pub(crate) amount: Option<f32>,
}
