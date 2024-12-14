use std::{env, sync::Arc};
use cnctd_server::{
    server::{CnctdServer, ServerConfig},
    socket::SocketConfig,
};
use local_ip_address::local_ip;
use router::{rest::RestRouter, socket::SocketRouter};
// use session::client_session::ClientSession;

pub mod router;
// pub mod db;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Load secrets and environment variables
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let server_id = env::var("SERVER_ID").unwrap_or_else(|_| "1".to_string());
    let port_str = env::var("SERVER_PORT").unwrap_or_else(|_| "5050".to_string());
    let client_dir = env::var("CLIENT_DIR").ok();
    let jwt_secret_bytes = jwt_secret.as_bytes().to_owned();

    // Initialize shared components
    router::rest::JWT_SECRET.set(jwt_secret.into());
    let rest_router = RestRouter;
    let socket_router = SocketRouter;

    // Allowed origins for CORS
    let ip_address = local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "127.0.0.1".to_string());
    let allowed_origins = vec![
        "http://localhost:3000".to_string(),
        "https://example.com".to_string(),
        format!("http://{}:{}", ip_address, port_str),
    ];

    // Server configuration
    let server_config = ServerConfig::new(
        &server_id,
        &port_str,
        client_dir,
        rest_router,
        Some(10),                  // Maximum concurrent connections
        Some(allowed_origins),     // Allowed CORS origins
        None,                      // Optional TLS config
    );

    // Socket configuration
    let socket_config = SocketConfig::new(
        socket_router,
        Some(jwt_secret_bytes),    // Secret for WebSocket auth
        None,    
        None,                  // Optional Redis URL
        // Some(Arc::new(|client_info| {
        //     let client_info_clone = client_info.clone();
        //     tokio::spawn(async move {
        //         let mut session = ClientSession::new(client_info_clone);
        //         if let Err(e) = session.disconnect().upload().await {
        //             println!("Error uploading session: {:?}", e);
        //         } else {
        //             println!("Session successfully uploaded.");
        //         }
        //     });
        // })),
    );

    // Start periodic session uploads
    // ClientSession::start_periodic_upload().await;

    // Initialize databases
    // if let Err(e) = db::DB::start().await {
    //     println!("Database error: {:?}", e);
    // } else {
    //     println!("Database initialized.");
    // }

    // Start the server
    if let Err(e) = CnctdServer::start(server_config, Some(socket_config)).await {
        println!("Server error: {:?}", e);
    } else {
        println!("Server started successfully.");
    }
}
