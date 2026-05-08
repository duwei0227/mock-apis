use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::net::UdpSocket;

#[derive(Serialize)]
pub struct ServerInfo {
    pub ip: String,
}

pub async fn get_info() -> impl IntoResponse {
    let ip = local_ip().unwrap_or_else(|| "127.0.0.1".to_owned());
    Json(ServerInfo { ip })
}

fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
