use std::{future::Future, pin::Pin};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use cnctd_server::{
    router::{message::{Message, SocketMessage}, SocketRouterFunction},
    socket::{client::CnctdClient, CnctdSocket},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToUser {
    pub channel: String,
    pub instruction: String,
    pub data: Option<Value>,
}

impl MessageToUser {
    pub fn new<T: Serialize>(channel: &str, instruction: &str, data: Option<T>) -> Self {
        Self {
            channel: channel.to_string(),
            instruction: instruction.to_string(),
            data: data.map(|d| serde_json::to_value(d).unwrap()),
        }
    }

    pub async fn send_to_client(&self, client_id: &str) -> anyhow::Result<()> {
        CnctdClient::message_client(client_id, self).await?;
        Ok(())
    }

    pub async fn send_to_user(&self, user_id: &str, exclude_client_id: Option<String>) -> anyhow::Result<()> {
        CnctdClient::message_user(user_id, self, exclude_client_id).await?;
        Ok(())
    }

    pub async fn send_to_subscribers(&self, channel: &str, exclude_client_id: Option<String>) -> anyhow::Result<()> {
        CnctdClient::message_subscribers(channel, self, exclude_client_id).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketResponse {
    pub success: bool,
    pub channel: String,
    pub instruction: String,
    pub data: Option<String>,
}

impl SocketResponse {
    pub fn success(data: Option<String>, channel: Option<String>) -> Self {
        Self {
            success: true,
            channel: channel.unwrap_or_else(|| "socket".to_string()),
            instruction: "success".to_string(),
            data,
        }
    }

    pub fn failure(data: Option<String>, channel: Option<String>) -> Self {
        Self {
            success: false,
            channel: channel.unwrap_or_else(|| "socket".to_string()),
            instruction: "failure".to_string(),
            data,
        }
    }
}

#[derive(Clone)]
pub struct SocketRouter;

impl SocketRouterFunction<SocketMessage, SocketResponse> for SocketRouter {
    fn route(&self, msg: SocketMessage, client_id: String) -> Pin<Box<dyn Future<Output = Option<SocketResponse>> + Send>> {
        Box::pin(async move { route_socket(msg, client_id).await })
    }
}

#[derive(Debug)]
pub enum SocketChannel {
    Ping,
    Broadcast,
    ClientData,
    Unrecognized,
}

impl SocketChannel {
    pub fn from_channel_str(s: &str) -> Self {
        match s {
            "ping" => SocketChannel::Ping,
            "broadcast" => SocketChannel::Broadcast,
            "client_data" => SocketChannel::ClientData,
            _ => SocketChannel::Unrecognized,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataIn {
    id: Option<String>,
    artist_id: Option<String>,
    name: Option<String>,
    image_id: Option<String>,
    recording_id: Option<String>,
}

async fn route_socket(msg: SocketMessage, client_id: String) -> Option<SocketResponse> {
    println!("Routing message...channel: {}, instruction: {:?}", msg.channel, msg.instruction);

    let data: DataIn = serde_json::from_value(msg.data.clone().unwrap_or(json!({})))
        .unwrap_or_else(|_| DataIn { id: None, artist_id: None, name: None, image_id: None, recording_id: None });

    match SocketChannel::from_channel_str(&msg.channel) {
        SocketChannel::Ping => Some(SocketResponse::success(Some("pong".to_string()), None)),

        SocketChannel::ClientData => {
            if let Some(artist_id) = data.artist_id {
                // Mock response to simulate some operation
                let mock_response = json!({
                    "artist_id": artist_id,
                    "info": "Mocked artist catalog data",
                });
                Some(SocketResponse::success(Some(mock_response.to_string()), None))
            } else {
                Some(SocketResponse::failure(Some("Missing artist ID".to_string()), None))
            }
        }

        SocketChannel::Broadcast => {
            if let Some(broadcast_msg) = msg.data {
                println!("Broadcasting message: {:?}", broadcast_msg);
                let parsed_msg: Message = serde_json::from_value(broadcast_msg)
                    .unwrap();
                CnctdSocket::broadcast_message(&parsed_msg).await.unwrap();
                Some(SocketResponse::success(None, None))
            } else {
                Some(SocketResponse::failure(Some("Invalid broadcast message".to_string()), None))
            }
        }

        _ => Some(SocketResponse::failure(Some("Unrecognized channel".to_string()), None)),
    }
}
