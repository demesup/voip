# VoIP Communication System - Project Documentation

## Project Overview

This is a **web-based Voice over IP (VoIP) communication system** that enables real-time voice communication between users through a browser interface. The system allows users to initiate voice calls, place calls on hold, mute audio, and manage active connections through an intuitive web interface.

The project implements peer-to-peer (P2P) voice communication using **WebRTC** for direct audio transmission, with a centralized **Rust backend** handling user management, signaling, and call state coordination.

---

## System Architecture

### High-Level Architecture

```
┌─────────────┐         HTTPS (Port 5000)          ┌──────────────┐
│   Browser   │◄──────────────────────────────────►│     Rust     │
│  (Client A) │         REST API + WebRTC          │    Backend   │
│             │         Signaling                  │   (Server)   │
└─────────────┘                                    └──────────────┘
      ▲                                                     ▲
      │                                                     │
      │  WebRTC P2P Audio (Direct)                        │
      │                                                     │
      ▼                                                     ▼
┌─────────────┐         HTTPS (Port 5000)          ┌──────────────┐
│   Browser   │◄──────────────────────────────────►│  Static File │
│  (Client B) │         REST API + WebRTC          │   Serving    │
│             │         Signaling                  │              │
└─────────────┘                                    └──────────────┘
```

### Technology Stack

#### Backend
- **Language**: Rust
- **Web Framework**: Actix-web 4.0
- **TLS/HTTPS**: rustls 0.20 with self-signed certificates
- **Async Runtime**: Tokio (full features)
- **Data Formats**: JSON (serde)
- **Static File Serving**: actix-files 0.6

#### Frontend
- **Languages**: HTML5, CSS3, JavaScript
- **WebRTC API**: Native browser WebRTC implementation
- **UI Framework**: JavaScript (no external dependencies)

#### Security
- **Protocol**: HTTPS only (required for WebRTC getUserMedia)
- **Certificates**: Self-signed TLS certificates (cert.pem/key.pem)
- **STUN Servers**: Google STUN servers for NAT traversal

---

## Features

### User Management
- **Registration**: Users register with a username to get a unique UUID
- **Online Status**: Real-time user list showing all connected users
- **Heartbeat System**: Periodic heartbeats (every 3 seconds) to maintain connection status
- **Auto-cleanup**: Inactive users removed after timeout

### Call Management
- **Initiate Calls**: Click-to-call any online user from the user list
- **Accept/Reject**: Incoming call modal with accept/decline options
- **Call Timer**: Real-time call duration display
- **Call Status**: Visual status indicators (Available, In Call, On Hold)

### Audio Controls
- **Hold/Resume**: 
  - Pause call and timer
  - Mute audio
  - Synchronized hold state between both users
- **Mute/Unmute**: Toggle local microphone independently
- **End Call**: Terminate active calls with cleanup

---

## System Components

### Backend Components (Rust)

#### `main.rs`
- **Purpose**: Entry point for the Actix-web server
- **Key Responsibilities**:
  - TLS/HTTPS configuration with certificate loading
  - Route registration for API endpoints
  - Static file serving for frontend assets
  - Server binding on port 5000
- **TLS Setup**:
  ```rust
  let config = load_rustls_config();
  HttpServer::new(...)
      .bind_rustls("0.0.0.0:5000", config)?
      .run()
      .await
  ```

#### `user.rs`
- **Purpose**: User management and state tracking
- **Data Structures**:
  - `User`: Contains username, UUID, status, last heartbeat, call ID
  - `UserManager`: Thread-safe (Arc<Mutex>) user registry
- **Key Functions**:
  - `register_user()`: Creates new user with UUID
  - `update_heartbeat()`: Updates user activity timestamp
  - `list_users()`: Returns all active users
  - `remove_inactive_users()`: Cleanup task

#### `call_manager.rs`
- **Purpose**: Call state management and coordination
- **Data Structures**:
  - `Call`: Contains call ID, participants, status (incall/onhold)
  - `CallManager`: Thread-safe call registry
- **Key Functions**:
  - `create_call()`: Initializes new call between two users
  - `end_call()`: Cleans up call state
  - `hold_call()`: Sets call status to "onhold"
  - `resume_call()`: Sets call status back to "incall"

#### `signaling.rs`
- **Purpose**: WebRTC signaling message relay
- **Key Functions**:
  - `send_offer()`: Relay SDP offer from caller to callee
  - `send_answer()`: Relay SDP answer from callee to caller
  - `send_ice_candidate()`: Exchange ICE candidates for NAT traversal
  - `check_incoming_call()`: Polling endpoint for incoming calls
  - `check_call_status()`: Returns current call state for synchronization

#### API Endpoints

**User Management**:
- `POST /api/users/register` - Register new user
- `GET /api/users/list` - Get all online users
- `POST /api/users/heartbeat` - Update user activity

**Call Initiation**:
- `POST /api/call/initiate` - Start a call
- `POST /api/call/accept` - Accept incoming call
- `POST /api/call/reject` - Reject incoming call
- `POST /api/call/end` - End active call

**Call Controls**:
- `POST /api/call/hold` - Put call on hold
- `POST /api/call/resume` - Resume held call

**Signaling**:
- `POST /api/signal/offer` - Send WebRTC offer
- `POST /api/signal/answer` - Send WebRTC answer
- `POST /api/signal/ice` - Exchange ICE candidates
- `GET /api/signal/incoming?user_id={id}` - Poll for incoming calls
- `GET /api/signal/status?call_id={id}` - Get call status

### State Management
```javascript
const appState = {
    userId: null,
    username: null,
    callId: null,
    peerConnection: null,
    localStream: null,
    remoteStream: null,
    isOnHold: false,
    isMuted: false,
    callStartTime: null,
    pausedDuration: 0,
    holdStartTime: null
};
```

##### WebRTC Connection Setup
- **`initiateCall()`**: 
  - Requests microphone access
  - Creates RTCPeerConnection
  - Generates SDP offer
  - Sends offer to peer via signaling server
- **`acceptCall()`**:
  - Requests microphone access
  - Creates RTCPeerConnection
  - Generates SDP answer
  - Sends answer to peer
- **`setupPeerConnectionHandlers()`**:
  - `onicecandidate`: Send ICE candidates
  - `ontrack`: Attach remote audio stream
  - `onconnectionstatechange`: Monitor connection state

##### Call Control Functions
- **`holdCall()`**:
  - Pauses call timer
  - Disables local audio tracks
  - Mutes remote audio element
  - Sends hold request to server
- **`resumeCall()`**:
  - Resumes call timer
  - Re-enables local audio tracks
  - Unmutes remote audio
  - Sends resume request to server
- **`muteCall()`**: Toggles local microphone track
- **`endCall()`**: Closes peer connection, stops streams, cleans up state

##### Polling and Synchronization
- **`checkIfCallEnded()`**: 
  - Polls server every 1 second during active calls
  - Syncs hold state from server to client
  - Detects when remote user ends call
  - Auto-cleanup when call terminates

##### Timer Management
```javascript
function updateCallTimer() {
    if (!appState.isOnHold && appState.callStartTime) {
        const elapsed = Math.floor((Date.now() - appState.callStartTime) / 1000);
        const minutes = Math.floor(elapsed / 60);
        const seconds = elapsed % 60;
    }
}
```

---


## Setup and Deployment

### Prerequisites
- **Rust**
- **Python 3**: For certificate generation (cryptography library)
- **Modern Browser**: Chrome 90+, Firefox 88+, Edge 90+, Safari 14+
- **Network**: Devices on same LAN or accessible via public IP

### Installation Steps

#### 1. Generate TLS Certificates

```powershell
cd \backend
python generate_cert.py
```

This creates:
- `cert.pem`: Self-signed TLS certificate
- `key.pem`: Private key

**Certificate Details**:
- Algorithm: RSA 2048-bit
- Validity: 365 days
- Subject Alternative Names: localhost, 127.0.0.1, local IP

#### 2. Build Backend

```powershell
cd \backend
cargo build --release
```

Compiled binary location: `target/release/voip-backend.exe`

#### 3. Run Backend Server

**Development mode**:
```powershell
cargo run
```

**Production mode**:
```powershell
.\target\release\voip-backend.exe
```

Server starts on: `https://0.0.0.0:5000`

#### 4. Access Frontend

**From same machine**:
```
https://localhost:5000
```

**From other devices on LAN**:
```
https://<server-ip>:5000
```
Example: `https://192.168.1.100:5000`

**Note**: Browser will show certificate warning (self-signed cert). Click "Advanced" → "Proceed to site" to continue.
