use std::time::Duration;

#[cfg(feature = "integration-test")]
use tokio::net::TcpStream;
use tokio::{
    io::BufWriter,
    sync::mpsc::Sender,
    time::{Instant, sleep},
};
use tokio_socks::tcp::Socks5Stream;
use tracing::{error, info, warn};

use crate::{
    error::TrackerError,
    handle_result,
    server::send_message_with_prefix,
    status,
    types::{DbRequest, ServerInfo, TrackerClientToServer, TrackerServerToClient},
    utils::read_message,
};

use tokio::io::BufReader;

const COOLDOWN_PERIOD: u64 = 15 * 60;
pub async fn monitor_systems(
    db_tx: Sender<DbRequest>,
    status_tx: status::Sender,
    #[cfg(not(feature = "integration-test"))] socks_port: u16,
    onion_address: String,
    port: u16,
) -> Result<(), TrackerError> {
    info!("Starting to monitor other maker services");

    loop {
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(1);
        if db_tx.send(DbRequest::QueryAll(response_tx)).await.is_err() {
            continue;
        }

        if let Some(response) = response_rx.recv().await {
            for (address, server_info) in response {
                let cooldown_duration = Duration::from_secs(COOLDOWN_PERIOD);
                if server_info.cooldown.elapsed() <= cooldown_duration {
                    continue;
                }
                info!("Address to query: {:?}", address);

                let mut success = false;
                for attempt in 1..=3 {
                    #[cfg(not(feature = "integration-test"))]
                    let connect_result = Socks5Stream::connect(
                        format!("127.0.0.1:{socks_port:?}").as_str(),
                        address.clone(),
                    )
                    .await;

                    #[cfg(feature = "integration-test")]
                    let connect_result = TcpStream::connect(address.clone()).await;

                    match connect_result {
                        Ok(mut stream) => {
                            success = true;

                            let (read_half, write_half) = stream.split();

                            let mut reader = BufReader::new(read_half);

                            let mut writer = BufWriter::new(write_half);

                            let message = TrackerServerToClient::Ping {
                                address: onion_address.clone(),
                                port,
                            };
                            _ = send_message_with_prefix(&mut writer, &message).await;

                            let Ok(buffer) = read_message(&mut reader).await else {
                                continue;
                            };
                            let response: TrackerClientToServer =
                                match serde_cbor::de::from_reader(&buffer[..]) {
                                    Ok(resp) => resp,
                                    Err(e) => {
                                        error!("Deserialization error: {e:?}");
                                        sleep(Duration::from_secs(1)).await;
                                        continue;
                                    }
                                };

                            if let TrackerClientToServer::Pong { address } = response {
                                let updated_info = ServerInfo {
                                    onion_address: address.clone(),
                                    cooldown: Instant::now(),
                                    stale: false,
                                };
                                let _ = db_tx.send(DbRequest::Update(address, updated_info)).await;
                            }

                            break;
                        }

                        Err(e) => {
                            warn!(
                                "Failed to connect to {} (attempt {}/3): {}",
                                address, attempt, e
                            );
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }

                if !success && !server_info.stale {
                    let updated_info = ServerInfo {
                        stale: true,
                        ..server_info
                    };
                    let _ = db_tx.send(DbRequest::Update(address, updated_info)).await;
                }
            }
            sleep(Duration::from_secs(COOLDOWN_PERIOD)).await;
        }
    }
}
