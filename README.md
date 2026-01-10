# VoIP Application - Complete Setup Guide

## Project Overview

A modern Voice over IP (VoIP) application with:
- **Backend**: Rust with Actix-web framework
- **Frontend**: HTML5, CSS3, JavaScript with WebRTC
- **Features**: Call initiation, status tracking, audio transmission, mute/hold controls

## Project Structure

```
voip-app/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # Main server & HTTP endpoints
│       ├── user.rs           # User & CallStatus structs
│       ├── call_manager.rs   # Call management logic
│       └── signaling.rs      # WebSocket signaling handlers
└── frontend/
    ├── index.html            # Main UI
    ├── styles.css            # Styling
    └── app.js                # Application logic
```

## Features

### Call Status Display
- **IDLE** - No active call
- **CALLING** - Outgoing call in progress
- **IN CALL** - Active call established
- **ON HOLD** - Call paused

### Call Management
- ✅ Direct user calling from the user list
- ✅ Call by IP address input
- ✅ Call rejection with confirmation dialog
- ✅ Incoming call notifications
- ✅ Hold/Resume functionality

### Audio Features
- ✅ Real-time audio capture (with permission)
- ✅ Audio level visualization
- ✅ Mute/Unmute button
- ✅ Frequency spectrum visualization
- ✅ Call duration timer

## Installation & Setup

### Prerequisites
- Rust 1.70+ (for backend)
- Node.js/npm or Python (for serving frontend)
- Modern web browser (Chrome, Firefox, Edge)

### Backend Setup

1. Navigate to the backend directory:
```bash
cd backend
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run --release
```

The server will start on `http://localhost:8080`

### Frontend Setup

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Serve the frontend using Python:
```bash
python -m http.server 3000
```

Or using Node.js (if you have `http-server` installed):
```bash
npx http-server -p 3000
```

3. Open your browser and navigate to:
```
http://localhost:3000
```

## How to Use

### Starting a Call

1. **Register**: When you open the app, enter your username
2. **Select User**: Choose from the "Available Users" list
3. **Initiate Call**: Click the "Call" button or enter an IP address and click "Call by IP"
4. **Accept/Reject**: The callee will see an incoming call dialog
5. **End Call**: Click the "End Call" button to disconnect

### Call Controls

- **Mute Button**: Toggle microphone on/off
- **Hold Button**: Pause the call (preserves connection)
- **End Call Button**: Terminate the call
- **Call Timer**: Shows duration of the active call

## API Endpoints

### User Management
- `POST /api/users/register` - Register a new user
- `GET /api/users/list` - List all registered users

### Call Signaling
- `POST /api/signal/initiate` - Initiate a new call
- `POST /api/signal/accept` - Accept an incoming call
- `POST /api/signal/reject` - Reject an incoming call
- `POST /api/signal/end` - End an active call
- `POST /api/signal/hold` - Put call on hold
- `POST /api/signal/resume` - Resume a held call

### Health Check
- `GET /api/health` - Server health status

## Audio Transmission

The application uses:
- **Browser Audio API**: For capturing and processing audio
- **WebRTC**: For peer-to-peer audio transmission (in production)
- **Audio Context**: For real-time audio analysis and visualization

### Current Implementation
- Local microphone capture with user permission
- Audio level meter visualization
- Real-time frequency spectrum display
- Call duration tracking

### Production Enhancements
- Implement WebRTC for actual peer-to-peer audio
- Add STUN/TURN servers for NAT traversal
- Implement RTP protocol for audio streaming
- Add codec support (Opus, G.711, etc.)

## Configuration

### Server Configuration
Edit `backend/src/main.rs` to change:
- Server address (default: 0.0.0.0:8080)
- CORS settings
- Logging level

### Frontend Configuration
Edit `frontend/app.js`:
- `API_BASE` - Backend server address (default: http://localhost:8080/api)
- Audio constraints in `requestAudioPermission()`
- Poll intervals for user updates

## Troubleshooting

### "Connection refused" error
- Ensure the Rust backend is running on port 8080
- Check firewall settings

### Microphone not working
- Grant microphone permission when prompted
- Check browser privacy settings
- Verify microphone is not in use by another application

### Users not appearing
- Ensure you've registered at least 2 users
- Refresh the user list (automatic every 5 seconds)
- Check browser console for errors

### Call not connecting
- Both users must be registered
- Check network connectivity
- Ensure both are on the same network or use IP address

## Development Notes

### Adding WebRTC Support
To implement actual audio transmission:

1. Install WebRTC dependencies in Rust:
```toml
webrtc = "0.9"
tokio-util = "0.7"
```

2. Implement SDP offer/answer exchange in signaling handlers
3. Set up data channels for audio frames

### Extending Features
- Add call history logging
- Implement call recording
- Add call transfer functionality
- Create user profiles/avatars
- Implement conference calling

## Performance Considerations

- Frontend updates users every 5 seconds (configurable)
- Audio visualization runs at 60 FPS
- Call timer updates every second
- Rust async runtime handles concurrent connections

## Security Notes

In production:
- Add authentication/authorization
- Use HTTPS/WSS for encryption
- Implement rate limiting
- Validate all user inputs
- Add CORS configuration
- Use certificate pinning for WebRTC

## License

MIT License - Feel free to use and modify

## Support

For issues or questions, check:
- Browser console (F12) for JavaScript errors
- Server logs (backend terminal)
- Network tab in DevTools for API calls
