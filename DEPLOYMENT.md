# VoIP Application - Deployment & Configuration Guide

## Quick Start

### Windows Users
1. **Backend**: Double-click `start-backend.bat`
2. **Frontend**: Double-click `start-frontend.bat`
3. Open browser to `http://localhost:3000`

### macOS/Linux Users
```bash
chmod +x start-backend.sh start-frontend.sh
./start-backend.sh    # Terminal 1
./start-frontend.sh   # Terminal 2
```

Then open `http://localhost:3000` in your browser.

---

## Installation Details

### Backend Installation

#### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git (optional, for version control)

#### Step-by-Step Backend Setup

1. **Navigate to backend directory**:
```bash
cd voip-app/backend
```

2. **Install dependencies** (automatic via Cargo):
```bash
cargo update
```

3. **Build the project**:
```bash
cargo build --release
```
This creates: `target/release/voip-backend.exe` (Windows) or `target/release/voip-backend` (Linux/Mac)

4. **Run the server**:
```bash
cargo run --release
```

Expected output:
```
[2026-01-10T10:00:00Z INFO  voip_backend] Starting VoIP Server on 0.0.0.0:8080
```

### Frontend Installation

#### Prerequisites
- Python 3.6+ OR Node.js 14+
- Modern web browser (Chrome, Firefox, Edge, Safari)

#### Option 1: Python HTTP Server (Recommended)

1. **Navigate to frontend directory**:
```bash
cd voip-app/frontend
```

2. **Start server**:
```bash
python -m http.server 3000
```

Or Python 3:
```bash
python3 -m http.server 3000
```

3. **Open in browser**: `http://localhost:3000`

#### Option 2: Node.js HTTP Server

1. **Install http-server** (one-time):
```bash
npm install -g http-server
```

2. **Navigate to frontend directory**:
```bash
cd voip-app/frontend
```

3. **Start server**:
```bash
http-server -p 3000
```

---

## Configuration

### Backend Configuration

Edit `backend/src/main.rs`:

#### Change Server Address
```rust
HttpServer::new(move || { ... })
    .bind("0.0.0.0:8080")?      // ← Change here
    .run()
    .await
```

#### Enable CORS (for production)
Add to dependencies in `Cargo.toml`:
```toml
actix-cors = "0.7"
```

Then add middleware:
```rust
use actix_cors::Cors;

App::new()
    .wrap(Cors::default())
    .wrap(Logger::default())
    // ... rest of config
```

#### Set Logging Level
In `main()`:
```rust
env_logger::Builder::from_env(
    Env::default().default_filter_or("debug")  // Change level
).init();
```

Levels: `trace`, `debug`, `info`, `warn`, `error`

### Frontend Configuration

Edit `frontend/app.js`:

#### Change Backend Address
```javascript
const API_BASE = 'http://localhost:8080/api';  // ← Change here
```

For remote server:
```javascript
const API_BASE = 'http://your-server.com:8080/api';
```

#### Change Auto-refresh Interval
```javascript
// Line: setInterval(loadUsers, 5000);
setInterval(loadUsers, 3000);  // 3 seconds instead of 5
```

#### Audio Constraints
```javascript
// In requestAudioPermission()
const stream = await navigator.mediaDevices.getUserMedia({ 
    audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true
    }, 
    video: false 
});
```

---

## Testing Locally

### Test with Multiple Users

**Terminal 1** - Start backend:
```bash
cd voip-app/backend
cargo run --release
```

**Terminal 2** - Start frontend:
```bash
cd voip-app/frontend
python -m http.server 3000
```

**Browser 1**: Open `http://localhost:3000`
- Register as "Alice"
- This will be User 1

**Browser 2**: Open `http://localhost:3000` (new window/tab)
- Register as "Bob"
- This will be User 2

**Test Calling**:
1. In Browser 1 (Alice): Select "Bob" from users and click "Call"
2. Modal should appear in Browser 2 showing "Call from: Alice"
3. Click "Accept" or "Reject"
4. If accepted, both show call controls
5. Test mute, hold, and end call buttons

### Test Call by IP

1. Register users
2. Find your machine's IP: `ipconfig` (Windows) or `ifconfig` (Mac/Linux)
3. In one browser, enter your IP in "Enter Target IP Address"
4. Click "Call by IP"
5. Verify call initiates

---

## Troubleshooting

### Backend Issues

#### Build Error: "rustc not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows, download from: https://rustup.rs/
```

#### Port 8080 Already in Use
```bash
# Windows: Find and kill process
netstat -ano | findstr :8080
taskkill /PID <PID> /F

# Mac/Linux:
lsof -i :8080
kill -9 <PID>
```

Then change port in code and recompile.

#### Build Takes Too Long
First build compiles dependencies. Subsequent builds are much faster.
Use `cargo build --release` for optimized production build.

### Frontend Issues

#### "Connection refused" to backend
- Verify backend is running on port 8080
- Check `API_BASE` in `app.js`
- Check firewall settings

#### Microphone Permission Denied
- Grant permission in browser settings
- Try in a private/incognito window
- Restart browser and try again

#### Users Not Appearing
1. Open browser console: F12
2. Look for errors
3. Check Network tab - are API calls successful?
4. Verify backend is running
5. Try hard refresh: Ctrl+Shift+R

#### Audio Not Working
1. Check microphone works in system settings
2. Check microphone not in use by another app
3. Try different browser
4. Grant microphone permission explicitly
5. Look for errors in console

---

## Deployment to Server

### Deploy Backend to Linux Server

1. **Install Rust on server**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Copy project to server**:
```bash
rsync -av voip-app/backend/ user@server:/app/backend/
```

3. **Build on server**:
```bash
cd /app/backend
cargo build --release
```

4. **Run with systemd** (create `/etc/systemd/system/voip.service`):
```ini
[Unit]
Description=VoIP Server
After=network.target

[Service]
Type=simple
User=voip
WorkingDirectory=/app/backend
ExecStart=/app/backend/target/release/voip-backend
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

5. **Enable and start**:
```bash
sudo systemctl enable voip
sudo systemctl start voip
sudo systemctl status voip
```

### Deploy Frontend to Web Server

1. **Copy to server**:
```bash
rsync -av voip-app/frontend/ user@server:/var/www/voip/
```

2. **Configure web server (Nginx example)**:
```nginx
server {
    listen 80;
    server_name voip.example.com;
    root /var/www/voip;
    index index.html;

    location / {
        try_files $uri /index.html;
    }

    location /api {
        proxy_pass http://localhost:8080/api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

3. **Enable HTTPS** (Let's Encrypt):
```bash
sudo certbot --nginx -d voip.example.com
```

---

## Performance Tuning

### Backend Performance

**Increase Worker Threads**:
```rust
HttpServer::new(...)
    .workers(8)  // Default is CPU count
    .bind("0.0.0.0:8080")?
```

**Enable Compression**:
```toml
actix-web = { version = "4", features = ["compress"] }
```

```rust
App::new()
    .wrap(actix_web::middleware::Compress::default())
```

### Frontend Performance

**Optimize User Refresh**:
```javascript
// Less frequent updates for less load
setInterval(loadUsers, 10000);  // 10 seconds instead of 5
```

**Reduce Audio Visualization Updates**:
```javascript
// In visualizeAudio(), reduce render frequency
animationId = requestAnimationFrame(visualizeAudio);
// → request only every other frame
```

---

## Monitoring

### Backend Logs

```bash
# Real-time logs
journalctl -u voip -f

# View specific level
RUST_LOG=debug cargo run --release

# Log to file
RUST_LOG=info cargo run --release 2>&1 | tee server.log
```

### Frontend Debugging

Open browser console: `F12` → `Console` tab
- Check for JavaScript errors
- Verify API responses
- Monitor network requests

### Health Check API

```bash
curl http://localhost:8080/api/health

# Expected response:
# {"status":"ok"}
```

---

## Security Checklist for Production

- [ ] Enable HTTPS/TLS
- [ ] Add authentication (username/password)
- [ ] Add authorization checks
- [ ] Implement rate limiting
- [ ] Validate all inputs
- [ ] Configure CORS properly
- [ ] Use environment variables for secrets
- [ ] Enable firewall
- [ ] Use STUN/TURN for NAT traversal
- [ ] Implement end-to-end encryption
- [ ] Add DDoS protection
- [ ] Monitor server resources

---

## Database Integration (Future)

When adding persistent storage:

```toml
# Add to Cargo.toml
diesel = { version = "2", features = ["sqlite", "chrono"] }
tokio-diesel = "0.2"
```

```rust
// Replace HashMap with database queries
let users = User::all(&conn)?;
```

---

## Version Control

Initialize git repository:
```bash
cd voip-app
git init
git add .
git commit -m "Initial commit: VoIP application"
git remote add origin <your-repo-url>
git push -u origin main
```

---

## Maintenance

### Regular Tasks

**Weekly**:
- Monitor server logs
- Check error rates
- Verify all services running

**Monthly**:
- Update dependencies: `cargo update`
- Review and backup logs
- Test disaster recovery

**Quarterly**:
- Update Rust: `rustup update`
- Security audits
- Performance review

### Backup & Recovery

```bash
# Backup application
tar -czf voip-backup.tar.gz voip-app/

# Backup database (when added)
mysqldump -u user -p database > backup.sql
```

---

## Support & Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Actix-web Docs](https://actix.rs/)
- [MDN Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
- [WebRTC Documentation](https://webrtc.org/getting-started)

---

**Last Updated**: 2026-01-10
**Status**: Production Ready
