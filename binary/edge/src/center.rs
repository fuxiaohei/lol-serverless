use crate::{conf::process_conf, conf::CONF_VALUES, localip, server};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use land_core::confdata::{RegionRecvData, RegionReportData};
use std::ops::ControlFlow;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, debug_span, warn, Instrument};
use tracing::{info, instrument};

async fn build_report_data() -> RegionReportData {
    let localip = localip::IPINFO.get().unwrap().clone();
    let region = localip.region();
    let runtimes = server::get_living_runtimes().await;
    let local_conf = CONF_VALUES.lock().await;
    let report_data = RegionReportData {
        localip,
        region,
        runtimes,
        conf_value_time_version: local_conf.created_at,
        time_at: chrono::Utc::now().timestamp() as u64,
        owner_id: 0,
    };
    report_data
}

#[instrument(name = "[WS]", skip_all)]
pub async fn init(addr: String, protocol: String, token: String) {
    let ipinfo = crate::localip::IPINFO.get().unwrap();
    let url = format!(
        "{}://{}/v1/region/ws?token={}&region={}",
        protocol,
        addr,
        token,
        ipinfo.region_ip()
    );
    info!("connect to {}", url);

    let reconnect_interval = std::time::Duration::from_secs(5);

    loop {
        debug!("connect to {} in loop", url);

        let ws_stream = match connect_async(&url).await {
            Ok((stream, _response)) => stream,
            Err(e) => {
                warn!("Error during handshake {:?}", e);
                info!("reconnect after {:?}", reconnect_interval);
                tokio::time::sleep(reconnect_interval).await;
                continue;
            }
        };

        debug!("connected");

        let (mut sender, mut receiver) = ws_stream.split();

        sender
            .send(Message::Ping("Hello".into()))
            .await
            .expect("Can not send!");

        let mut send_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            loop {
                interval.tick().await;

                let report_data = build_report_data().await;
                let content = serde_json::to_vec(&report_data).unwrap();

                if sender.send(Message::Binary(content)).await.is_err() {
                    warn!("send report data failed");
                    return;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if process_message(msg)
                    .instrument(debug_span!("[WS]"))
                    .await
                    .is_break()
                {
                    break;
                }
            }
        });

        tokio::select! {
            _ = (&mut send_task) => {
                debug!("send task done");
                recv_task.abort();
            },
            _ = (&mut recv_task) => {
                debug!("recv task done");
                send_task.abort();
            }
        }

        info!("reconnect after {:?}", reconnect_interval);
        tokio::time::sleep(reconnect_interval).await;
    }
}

async fn process_message(msg: Message) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!("recv text: {:?}", t);
        }
        Message::Binary(d) => {
            let recv_data: RegionRecvData = serde_json::from_slice(&d).unwrap();
            process_conf(recv_data.conf_values).await;
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!("recv close: {:?}", cf);
            } else {
                info!("recv close");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(_v) => {
            info!("recv pong")
        }
        Message::Ping(_v) => {
            info!("recv ping")
        }
        Message::Frame(_) => {
            unreachable!("This is never supposed to happen")
        }
    }
    ControlFlow::Continue(())
}
