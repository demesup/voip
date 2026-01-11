use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex as StdMutex},
};

use tokio::{
    net::UdpSocket,
    sync::{
        broadcast::{self, Receiver as BroadcastReceiver},
        mpsc,
        mpsc::Receiver as SingleReceiver,
        Mutex,
    },
    task::JoinHandle,
};

use tokio_util::sync::CancellationToken;

use crate::{
    call_manager::CallManager,
    io::AudioState,
    jitter::JitterBuffer,
    packet::AudioPacket,
};

struct CallHandler {
    cancel_token: CancellationToken,
    send_handle: JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct UdpCommand {
    #[allow(dead_code)]
    pub user_id: String,
    pub command: String,
    pub target_ip: Option<IpAddr>,
}

pub async fn udp_audio_task(
    _call_manager: Arc<Mutex<CallManager>>,
    mut control_channel: SingleReceiver<UdpCommand>,
) -> Result<(), Box<dyn Error>> {
    let (tx_audio, _rx_audio) = broadcast::channel::<Vec<u8>>(128);
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:40000").await?);
    let local_ip = socket.local_addr()?.ip();

    log::info!("UDP audio server listening on {}:40000", local_ip);

    let mut call_handler: Option<CallHandler> = None;
    let mut caller_ip: Option<IpAddr> = None;
    let jitter = Arc::new(StdMutex::new(JitterBuffer::new()));

    let (tx_caller, mut rx_caller) = mpsc::channel::<IpAddr>(1);

   
    {
        let socket_recv = socket.clone();
        let jitter_recv = jitter.clone();
        let tx_caller_recv = tx_caller.clone();

        tokio::spawn(async move {
            let _ = receive_task(
                socket_recv,
                jitter_recv,
                CancellationToken::new(),
                tx_caller_recv,
                local_ip,
            )
            .await;
        });
    }

    loop {
        tokio::select! {
            Some(ip) = rx_caller.recv() => {
                log::info!("Detected caller IP: {}", ip);
                caller_ip = Some(ip);
            }

            msg = control_channel.recv() => {
                if let Some(cmd) = msg {
                    log::info!("Received UDP command: {:?}", cmd);

                    match cmd.command.as_str() {
                        "ping" => {
                            if let Some(ip) = cmd.target_ip {
                                let addr = SocketAddr::new(ip, 40000);
                                let packet = AudioPacket { seq: 0, samples: vec![] };
                                let _ = socket.send_to(&packet.serialize(), addr).await;
                                log::info!("Sent ping to {}", addr);
                            }
                        }
                        "start_call" => {
                            if let Some(target_ip) = caller_ip.or(cmd.target_ip) {
                                let target_addr = SocketAddr::new(target_ip, 40000);
                                log::info!("Starting call with target {}", target_addr);

                                let cancel_token = CancellationToken::new();
                                let audio_rx = tx_audio.subscribe();

                                let send_handle = {
                                    let token = cancel_token.clone();
                                    let socket_send = socket.clone();
                                    let target = target_addr;
                                    tokio::spawn(async move {
                                        let _ = send_task(socket_send, audio_rx, token, target).await;
                                    })
                                };

                                call_handler = Some(CallHandler {
                                    cancel_token,
                                    send_handle,
                                });

                               
                                let tx_audio_clone = tx_audio.clone();
                                let jitter_clone = jitter.clone();
                                std::thread::spawn(move || {
                                    let host = cpal::default_host();
                                    let mut audio_state = AudioState::new(host);
                                    audio_state.start(tx_audio_clone, jitter_clone);
                                   
                                    loop {
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                    }
                                });

                                log::info!("Audio streams started");
                            } else {
                                log::warn!("Cannot start call: no target IP available");
                            }
                        }
                        "end_call" => {
                            caller_ip = None;

                            if let Some(call) = call_handler.take() {
                                call.cancel_token.cancel();
                                let _ = call.send_handle.await;
                                log::info!("Call ended");
                            }
                        }
                        _ => {
                            log::warn!("Unknown UDP command: {}", cmd.command);
                        }
                    }
                }
            }
        }
    }
}

async fn send_task(
    socket: Arc<UdpSocket>,
    mut audio_channel: BroadcastReceiver<Vec<u8>>,
    cancel_token: CancellationToken,
    target_addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut packet_count = 0u64;
    let mut last_log = std::time::Instant::now();

    loop {
        if cancel_token.is_cancelled() {
            break;
        }

        match audio_channel.try_recv() {
            Ok(data) => {
                if let Some(packet) = AudioPacket::deserialize(&data) {
                   
                    let max_sample = packet.samples.iter().map(|s| s.abs()).max().unwrap_or(0);
                    let is_silence = max_sample < 100;

                    packet_count += 1;

                   
                    if last_log.elapsed().as_secs() >= 1 {
                        log::info!("📤 Sent {} packets to {} | Seq: {} | Samples: {} | Max: {} | {}",
                            packet_count,
                            target_addr,
                            packet.seq,
                            packet.samples.len(),
                            max_sample,
                            if is_silence { "🔇 SILENCE" } else { "🔊 AUDIO" }
                        );
                        packet_count = 0;
                        last_log = std::time::Instant::now();
                    }
                }
                let _ = socket.send_to(&data, target_addr).await;
            }
            Err(_) => {
               
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        }
    }
    Ok(())
}

async fn receive_task(
    socket: Arc<UdpSocket>,
    jitter: Arc<StdMutex<JitterBuffer>>,
    cancel_token: CancellationToken,
    tx_caller: mpsc::Sender<IpAddr>,
    local_ip: IpAddr,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 4096];
    let mut last_seq: u16 = 0;
    let mut packet_count = 0u64;
    let mut lost_packet_count = 0u64;
    let mut last_log = std::time::Instant::now();

    loop {
        tokio::select! {
            recv = socket.recv_from(&mut buf) => {
                if let Ok((size, addr)) = recv {
                   
                    if addr.ip() == local_ip {
                        continue;
                    }

                    if let Some(packet) = AudioPacket::deserialize(&buf[..size]) {
                        if packet.seq == 0 {
                           
                            log::info!("📡 Received ping from {}", addr);
                            let _ = tx_caller.send(addr.ip()).await;
                        } else {
                           
                            let max_sample = packet.samples.iter().map(|s| s.abs()).max().unwrap_or(0);
                            let is_silence = max_sample < 100;

                            packet_count += 1;

                           
                            if last_seq != 0 && packet.seq > last_seq + 1 {
                                let missing = (packet.seq - last_seq - 1) as usize;
                                lost_packet_count += missing as u64;
                                let silence = vec![0i16; missing * 960];
                                jitter.lock().unwrap().push_packet(&silence);
                                log::warn!("⚠️  Packet loss detected: {} packets missing (seq {} -> {})",
                                    missing, last_seq, packet.seq);
                            }

                            let buffer_size = {
                                let mut jb = jitter.lock().unwrap();
                                jb.push_packet(&packet.samples);
                                jb.buffer.len()
                            };

                            last_seq = packet.seq;

                           
                            if last_log.elapsed().as_secs() >= 1 {
                                let loss_rate = if packet_count > 0 {
                                    (lost_packet_count as f32 / (packet_count + lost_packet_count) as f32) * 100.0
                                } else {
                                    0.0
                                };

                                log::info!("📥 Received {} packets from {} | Seq: {} | Samples: {} | Max: {} | Buffer: {} | Loss: {:.1}% | {}",
                                    packet_count,
                                    addr,
                                    packet.seq,
                                    packet.samples.len(),
                                    max_sample,
                                    buffer_size,
                                    loss_rate,
                                    if is_silence { "🔇 SILENCE" } else { "🔊 AUDIO" }
                                );
                                packet_count = 0;
                                lost_packet_count = 0;
                                last_log = std::time::Instant::now();
                            }
                        }
                    } else {
                        log::warn!("❌ Failed to deserialize packet from {}", addr);
                    }
                }
            }
            _ = cancel_token.cancelled() => break,
        }
    }
    Ok(())
}
