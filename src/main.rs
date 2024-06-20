use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

enum KeyValueMessage {
    Value(String),
    GetValue(mpsc::Sender<KeyValueMessage>),
}

#[tokio::main]
async fn main() {
    let key_map: HashMap<String, String> =
        HashMap::from([("see2et".to_string(), "kawaii".to_string())]);
    let (tx, rx) = mpsc::channel::<KeyValueMessage>(32);
    tokio::spawn(async move {
        value_server(rx, key_map).await;
    });
    // ルーティングを新しく作成する。
    let app = Router::new().route("/get/:key", get(get_key).with_state(tx));

    //サーバーがリッスンするIPアドレスとポートを設定
    //IP: 0.0.0.0
    //Port: 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    //サーバーを起動
    axum::serve(listener, app).await.unwrap();
}

async fn get_key(
    Path(param): Path<String>,
    State(tx): State<mpsc::Sender<KeyValueMessage>>,
) -> String {
    let (val_tx, mut val_rx) = mpsc::channel::<KeyValueMessage>(32);
    tx.send(KeyValueMessage::Value(param)).await.unwrap();
    tx.send(KeyValueMessage::GetValue(val_tx)).await.unwrap();
    if let Some(msg) = val_rx.recv().await {
        match msg {
            KeyValueMessage::Value(value) => {
                format!("value: {}", value)
            }
            _ => {
                panic!();
            }
        }
    } else {
        panic!();
    }
}

async fn value_server(mut rx: mpsc::Receiver<KeyValueMessage>, map: HashMap<String, String>) {
    let mut value: String = "None".to_string();
    while let Some(msg) = rx.recv().await {
        match msg {
            KeyValueMessage::Value(rcv_val) => {
                if let Some(v) = map.get(&rcv_val) {
                    value = v.to_string();
                } else {
                    value = "This key doesn't match any keys.".to_string();
                }
            }
            KeyValueMessage::GetValue(tx) => {
                tx.send(KeyValueMessage::Value(value.clone()))
                    .await
                    .unwrap();
            }
        }
    }
}
