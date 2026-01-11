mod audio_udp;
mod call_manager;
mod io;
mod jitter;
mod packet;
mod signaling;
mod user;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use actix_files::Files;
use audio_udp::UdpCommand;
use call_manager::CallManager;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use env_logger::Env;
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Load TLS certificates
    let mut cert_file = BufReader::new(File::open("cert.pem").expect("cert.pem not found. Run generate_cert.py first."));
    let mut key_file = BufReader::new(File::open("key.pem").expect("key.pem not found. Run generate_cert.py first."));
    let cert_chain = certs(&mut cert_file).expect("Failed to load certs").into_iter().map(Certificate).collect::<Vec<_>>();
    let mut keys = pkcs8_private_keys(&mut key_file).expect("Failed to load keys").into_iter().map(PrivateKey).collect::<Vec<_>>();
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0)).expect("Failed to create TLS config");

    let call_manager = Arc::new(Mutex::new(CallManager::new()));

    // Create UDP command channel
    let (udp_tx, udp_rx) = mpsc::channel::<UdpCommand>(32);

    // Spawn UDP audio task
    let call_manager_clone = Arc::clone(&call_manager);
    log::info!("Spawning UDP audio task...");
    tokio::spawn(async move {
        log::info!("UDP audio task started");
        if let Err(e) = audio_udp::udp_audio_task(call_manager_clone, udp_rx).await {
            log::error!("UDP audio task failed: {}", e);
        }
    });
    log::info!("UDP audio task spawned");

    log::info!("Starting VoIP Server on 0.0.0.0:5000");
    
    HttpServer::new(move || {
        let call_manager = Arc::clone(&call_manager);
        let udp_tx_clone = udp_tx.clone();
        let udp_tx_clone2 = udp_tx.clone();

        App::new()
            .app_data(web::Data::new(call_manager))
            .app_data(web::Data::new(udp_tx_clone))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health_check))
                    .route("/users/register", web::post().to(register_user))
                    .route("/users/list", web::get().to(list_users))
                    .route("/users/get", web::get().to(get_user))
                    .route("/users/disconnect", web::post().to(disconnect_user))
                    .route("/users/heartbeat", web::post().to(user_heartbeat))
                    .service(
                        web::scope("")
                            .app_data(udp_tx_clone2)
                            .configure(signaling::config_with_udp_sender)
                    )
            )
            .service(Files::new("/", "../frontend").index_file("index.html"))
    })
    .bind_rustls("0.0.0.0:5000", config)?
    .run()
    .await
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

async fn register_user(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    user_data: web::Json<serde_json::Value>,
) -> actix_web::HttpResponse {
    let user_id = uuid::Uuid::new_v4().to_string();
    let username = user_data
        .get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("Unknown");
    
    let mut manager = call_manager.lock().await;
    manager.register_user(user_id.clone(), username.to_string());
    
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_id,
        "username": username
    }))
}

async fn list_users(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
) -> actix_web::HttpResponse {
    let manager = call_manager.lock().await;
    let users = manager.list_users();
    
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "users": users
    }))
}

async fn disconnect_user(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    user_data: web::Json<serde_json::Value>,
) -> actix_web::HttpResponse {
    let user_id = user_data
        .get("user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("");
    
    let mut manager = call_manager.lock().await;
    let success = manager.disconnect_user(user_id);
    
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "success": success
    }))
}

async fn user_heartbeat(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    user_data: web::Json<serde_json::Value>,
) -> actix_web::HttpResponse {
    let user_id = user_data
        .get("user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("");
    
    let mut manager = call_manager.lock().await;
    
    // Update heartbeat for this user
    let user_exists = manager.update_heartbeat(user_id);
    
    // Check for inactive users (timeout after 10 seconds of no heartbeat)
    let _ = manager.disconnect_inactive_users(10);
    
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "success": user_exists
    }))
}


async fn get_user(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::HttpResponse {
    let user_id = query.get("user_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    
    if let Some(user) = manager.get_user(user_id) {
        actix_web::HttpResponse::Ok().json(serde_json::json!({
            "user_id": user.id,
            "username": user.username,
            "status": user.status
        }))
    } else {
        actix_web::HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        }))
    }
}
