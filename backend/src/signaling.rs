use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::audio_udp::UdpCommand;
use crate::call_manager::CallManager;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingMessage {
    pub message_type: String,
    pub user_id: String,
    pub call_id: Option<String>,
    pub target_user_id: Option<String>,
    pub offer: Option<String>,
    pub answer: Option<String>,
    pub candidate: Option<String>,
    pub ip_address: Option<String>,
}

pub fn config_with_udp_sender(cfg: &mut web::ServiceConfig) {
    cfg.route("/signal/initiate", web::post().to(initiate_call))
        .route("/signal/accept", web::post().to(accept_call))
        .route("/signal/reject", web::post().to(reject_call))
        .route("/signal/end", web::post().to(end_call))
        .route("/signal/hold", web::post().to(hold_call))
        .route("/signal/resume", web::post().to(resume_call))
        .route("/signal/incoming", web::get().to(check_incoming_calls))
        .route("/signal/status", web::get().to(get_call_status))
        .route("/signal/offer", web::post().to(send_offer))
        .route("/signal/answer", web::post().to(send_answer))
        .route("/signal/candidate", web::post().to(send_candidate))
        .route("/signal/get_offer", web::get().to(get_offer))
        .route("/signal/get_answer", web::get().to(get_answer))
        .route("/signal/get_candidates", web::get().to(get_candidates));
}

async fn initiate_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(target_id) = &msg.target_user_id {
        let call = manager.create_call(msg.user_id.clone(), target_id.clone());
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "call_id": call.call_id,
            "message": "Call initiated"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Target user ID required"
        }))
    }
}

async fn accept_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    udp_sender: web::Data<Sender<UdpCommand>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        manager.accept_call(call_id);
        
       
        if let Some(ip_str) = &msg.ip_address {
            if let Ok(target_ip) = ip_str.parse::<std::net::IpAddr>() {
                let udp_command = UdpCommand {
                    user_id: msg.user_id.clone(),
                    command: "start_call".to_string(),
                    target_ip: Some(target_ip),
                };
                
                if let Err(e) = udp_sender.send(udp_command).await {
                    log::error!("Failed to send UDP start command: {}", e);
                } else {
                    log::info!("Sent UDP start command to {}", target_ip);
                }
            } else {
                log::warn!("Invalid IP address in accept_call: {}", ip_str);
            }
        }
        
       
        if let Some(call) = manager.get_call(call_id) {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Call accepted",
                "call": {
                    "call_id": call.call_id,
                    "caller_id": call.caller_id,
                    "callee_id": call.callee_id,
                    "status": call.status
                }
            }))
        } else {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Call accepted"
            }))
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn reject_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        manager.reject_call(call_id);
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Call rejected"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn end_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    udp_sender: web::Data<Sender<UdpCommand>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        manager.end_call(call_id);
        
       
        if let Some(ip_str) = &msg.ip_address {
            if let Ok(target_ip) = ip_str.parse::<std::net::IpAddr>() {
                let udp_command = UdpCommand {
                    user_id: msg.user_id.clone(),
                    command: "end_call".to_string(),
                    target_ip: Some(target_ip),
                };
                
                if let Err(e) = udp_sender.send(udp_command).await {
                    log::error!("Failed to send UDP end command: {}", e);
                } else {
                    log::info!("Sent UDP end command to {}", target_ip);
                }
            } else {
                log::warn!("Invalid IP address in end_call: {}", ip_str);
            }
        }
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Call ended"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn hold_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        manager.hold_call(call_id);
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Call on hold"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn resume_call(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        manager.accept_call(call_id);
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Call resumed"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn check_incoming_calls(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let user_id = query.get("user_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    let calls = manager.get_incoming_calls(user_id);
    
    if let Some(call) = calls.first() {
        HttpResponse::Ok().json(serde_json::json!({
            "call": {
                "call_id": call.call_id,
                "caller_id": call.caller_id,
                "status": call.status
            }
        }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({
            "call": null
        }))
    }
}

async fn get_call_status(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let call_id = query.get("call_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    
    if let Some(call) = manager.get_call(call_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "call": {
                "call_id": call.call_id,
                "caller_id": call.caller_id,
                "callee_id": call.callee_id,
                "status": call.status
            }
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Call not found"
        }))
    }
}

async fn send_offer(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        if let Some(offer) = &msg.offer {
            manager.set_offer(call_id, offer.clone());
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Offer sent"
            }))
        } else {
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Offer required"
            }))
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn send_answer(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        if let Some(answer) = &msg.answer {
            manager.set_answer(call_id, answer.clone());
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Answer sent"
            }))
        } else {
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Answer required"
            }))
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn send_candidate(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    msg: web::Json<SignalingMessage>,
) -> HttpResponse {
    let mut manager = call_manager.lock().await;
    
    if let Some(call_id) = &msg.call_id {
        if let Some(candidate) = &msg.candidate {
            let is_caller = msg.user_id == manager.get_call(call_id).unwrap().caller_id;
            manager.add_candidate(call_id, candidate.clone(), is_caller);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Candidate sent"
            }))
        } else {
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Candidate required"
            }))
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Call ID required"
        }))
    }
}

async fn get_offer(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let call_id = query.get("call_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    
    if let Some(call) = manager.get_call(call_id) {
        if let Some(offer) = &call.offer {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "offer": offer
            }))
        } else {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "pending",
                "message": "Offer not yet available"
            }))
        }
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Call not found"
        }))
    }
}

async fn get_answer(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let call_id = query.get("call_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    
    if let Some(call) = manager.get_call(call_id) {
        if let Some(answer) = &call.answer {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "answer": answer
            }))
        } else {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "pending",
                "message": "Answer not yet available"
            }))
        }
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Call not found"
        }))
    }
}

async fn get_candidates(
    call_manager: web::Data<Arc<Mutex<CallManager>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let call_id = query.get("call_id").map(|s| s.as_str()).unwrap_or("");
    let user_id = query.get("user_id").map(|s| s.as_str()).unwrap_or("");
    
    let manager = call_manager.lock().await;
    
    if let Some(call) = manager.get_call(call_id) {
        let candidates = if user_id == call.caller_id {
            &call.callee_candidates
        } else {
            &call.caller_candidates
        };
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "candidates": candidates
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Call not found"
        }))
    }
}
