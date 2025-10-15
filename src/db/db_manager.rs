use crate::db::model::MempoolTx;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SqliteConnection, r2d2::ConnectionManager};
use r2d2::Pool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tracing::info;

use crate::{
    db::schema::{mempool_inputs, mempool_tx},
    error::TrackerError,
    status::{self, Status},
    types::{DbRequest, ServerInfo},
};

pub async fn run(
    pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
    mut rx: Receiver<DbRequest>,
    status_tx: status::Sender,
) {
    let mut servers: HashMap<String, ServerInfo> = HashMap::new();
    let mut conn = pool.get().unwrap();
    info!("DB manager started");
    while let Some(request) = rx.recv().await {
        match request {
            DbRequest::Add(addr, info) => {
                info!("Add request intercepted: address: {addr:?}, info: {info:?}");
                servers.insert(addr, info);
            }
            DbRequest::Query(addr, resp_tx) => {
                info!("Query request intecepted");
                let result = servers.get(&addr).cloned();
                let _ = resp_tx.send(result).await;
            }
            DbRequest::Update(addr, server_info) => {
                info!("Update request intercepted");
                servers.insert(addr, server_info);
            }
            DbRequest::QueryAll(resp_tx) => {
                info!("Query all request intercepted");
                let response: Vec<(String, ServerInfo)> =
                    servers.iter().map(|e| (e.0.clone(), e.1.clone())).collect();
                let _ = resp_tx.send(response).await;
            }
            DbRequest::QueryActive(resp_tx) => {
                info!("Query active intercepted");
                let response: Vec<String> = servers
                    .iter()
                    .filter(|x| !x.1.stale)
                    .map(|e| e.0.clone())
                    .collect();
                let _ = resp_tx.send(response).await;
            }
            DbRequest::WatchUtxo(outpoint, resp_tx) => {
                info!("Watch utxo intercepted");

                let mut mempool_tx = mempool_tx::table
                    .inner_join(mempool_inputs::table.on(mempool_tx::txid.eq(mempool_inputs::txid)))
                    .filter(mempool_inputs::input_txid.eq(outpoint.txid.to_string()))
                    .filter(mempool_inputs::input_vout.eq(outpoint.vout as i32))
                    .select((mempool_tx::txid, mempool_tx::seen_at))
                    .load::<MempoolTx>(&mut conn)
                    .unwrap();

                mempool_tx.sort_by(|a, b| a.txid.cmp(&b.txid));
                mempool_tx.dedup_by(|a, b| a.txid == b.txid);

                let _ = resp_tx.send(mempool_tx).await;
            }
        }
    }

    let _ = status_tx
        .send(Status {
            state: status::State::DBShutdown(TrackerError::DbManagerExited),
        })
        .await;
}
