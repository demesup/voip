# VoIP Backend - UDP Audio Transmission

This Rust backend implements real-time audio transmission over UDP for VoIP applications.

## Features

- **UDP Audio Transmission**: Send and receive audio packets over UDP
- **Jitter Buffer**: Handles network jitter and packet loss
- **Real-time Audio**: Low-latency audio capture and playback
- **WebRTC Signaling**: HTTP API for call management
- **Cross-platform**: Works on Windows, macOS, and Linux

## Architecture

### Components

- `main.rs`: Actix-web HTTP server for signaling
- `audio_udp.rs`: UDP audio handling with tokio async
- `io.rs`: Audio capture and playback using CPAL
- `jitter.rs`: Jitter buffer for smooth playback
- `packet.rs`: Audio packet serialization
- `signaling.rs`: WebRTC-style signaling over HTTP
- `call_manager.rs`: Call state management
- `user.rs`: User management

### Audio Flow

1. **Capture**: Microphone audio is captured in 20ms chunks (960 samples at 48kHz)
2. **Packetization**: Audio is serialized into UDP packets with sequence numbers
3. **Transmission**: Packets are sent over UDP to the target device
4. **Reception**: Packets are received and buffered in a jitter buffer
5. **Playback**: Audio is played back from the jitter buffer

## Usage

### Running the Full VoIP Server

```bash
cargo run --bin voip-backend
```

This starts:
- HTTP server on `0.0.0.0:5000` for signaling
- UDP server on `0.0.0.0:40000` for audio

### Testing UDP Audio

#### Send Test Audio

```bash
# Send to localhost
cargo run --bin udp-test

# Send to specific IP
cargo run --bin udp-test 192.168.1.100

# Send to specific IP and port
cargo run --bin udp-test 192.168.1.100 40000
```

#### Receive and Play Audio

```bash
cargo run --bin udp-receiver
```

This listens on UDP port 40000 and plays received audio.

### API Endpoints

- `POST /api/users/register` - Register a user
- `POST /api/signal/initiate` - Initiate a call
- `POST /api/signal/accept` - Accept a call
- `POST /api/signal/end` - End a call
- `GET /api/signal/incoming` - Check for incoming calls

## Packet Format

```
[sequence: u16][sample1: i16][sample2: i16]...[sample960: i16]
```

- **Sequence**: 16-bit packet sequence number
- **Samples**: 960 16-bit PCM samples (20ms at 48kHz)

## Jitter Buffer

- **Minimum Delay**: 100ms (4800 samples)
- **Maximum Delay**: 400ms (19200 samples)
- **Packet Loss Handling**: Inserts silence for missing packets

## Dependencies

- `tokio`: Async runtime
- `actix-web`: HTTP server
- `cpal`: Cross-platform audio
- `serde`: Serialization

## Building

```bash
cargo build --release
```

## Testing Between Devices

1. On Device A (sender):
   ```bash
   cargo run --bin udp-test <Device_B_IP>
   ```

2. On Device B (receiver):
   ```bash
   cargo run --bin udp-receiver
   ```

Make sure UDP port 40000 is open between devices.