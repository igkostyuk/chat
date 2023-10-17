use anyhow::Context;
use dioxus::prelude::*;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};

pub fn connect(
    url: &str,
) -> (
    Option<SplitSink<WebSocket, Message>>,
    Option<SplitStream<WebSocket>>,
) {
    match WebSocket::open(url) {
        Ok(ws) => {
            let ws = ws.split();
            (Some(ws.0), Some(ws.1))
        }
        Err(e) => {
            log::error!("failed to connect to socket:{}", e);
            (None, None)
        }
    }
}

pub async fn read<T: for<'a> Deserialize<'a>>(
    receiver: Option<SplitStream<WebSocket>>,
    handler: impl Fn(T),
) {
    if let Some(mut receiver) = receiver {
        while let Some(Ok(Message::Text(s))) = receiver.next().await {
            log::debug!("received:{}", &s);
            if let Ok(event) = serde_json::from_str::<T>(&s) {
                handler(event)
            }
        }
    }
}

pub async fn write<M: Serialize>(
    mut rx: UnboundedReceiver<M>,
    sender: Option<SplitSink<WebSocket, Message>>,
) {
    if let Some(mut sender) = sender {
        while let Some(msg) = rx.next().await {
            match serde_json::to_string(&msg) {
                Ok(msg) => {
                    log::debug!("sending:{}", &msg);
                    if let Err(e) = sender.send(Message::Text(msg)).await {
                        log::error!("failed send:{}", e);
                    }
                }
                Err(e) => log::error!("failed parse:{}", e),
            }
        }
    }
}
