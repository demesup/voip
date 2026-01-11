# VoIP App - VirtualBox Setup Guide

## 🚀 **Quick Start for VirtualBox VMs**

### **Network Configuration**

#### **Option 1: Bridged Networking (Recommended)**
1. **Shutdown your VMs**
2. **Change network adapter**:
   - VM Settings → Network → Attached to: **Bridged Adapter**
   - Name: Your host's network adapter
3. **Start VMs** - they'll get IP addresses from your router
4. **Find VM IPs**: `ip addr show` (Linux) or `ipconfig` (Windows)

#### **Option 2: Host-Only + NAT (Alternative)**
1. **Create Host-Only network**:
   - VirtualBox → File → Preferences → Network → Host-only Networks
   - Add new network (vboxnet0)
2. **VM1**: NAT adapter (for internet)
3. **VM2**: Host-only adapter (vboxnet0)
4. **Host**: Run backend + frontend servers
5. **VMs**: Access via host IP (usually 192.168.56.1)

### **Browser Requirements**

#### **✅ Supported Browsers**
- **Google Chrome** (recommended)
- **Mozilla Firefox**
- **Microsoft Edge**
- **Chromium-based browsers**

#### **❌ Not Supported**
- Older browsers without WebRTC
- Text-based browsers (lynx, w3m)
- Mobile browsers in some cases

### **Setup Steps**

#### **1. On Host Machine (Windows)**
```bash
# Start backend server
cd voip/backend
cargo run --bin voip-backend

# In another terminal, start frontend server
cd voip/frontend
python -m http.server 3000
```

#### **2. On VM Machines (Debian)**
```bash
# Install a modern browser
sudo apt update
sudo apt install chromium-browser

# Access the app
chromium-browser http://[HOST_IP]:3000
```

### **Troubleshooting**

#### **"navigator.mediaDevices is undefined"**
- **Cause**: Browser doesn't support WebRTC
- **Fix**: Install Chrome/Firefox, or use bridged networking

#### **"Failed to initiate call"**
- **Cause**: Backend not accessible or network issues
- **Fix**:
  - Check backend is running: `curl http://[BACKEND_IP]:5000/api/users/list`
  - Verify firewall: `sudo ufw allow 5000`
  - Check VirtualBox networking

#### **"Cannot connect to backend"**
- **Cause**: Wrong API_BASE URL
- **Fix**: The app auto-detects, but you can check browser console

#### **Audio Not Working**
- **Cause**: VM audio passthrough issues
- **Fix**: Enable audio in VM settings, or use UDP-only mode

### **Network Testing**

#### **Test Backend Connectivity**
```bash
# From VM, test backend access
curl http://[HOST_IP]:5000/api/users/list
```

#### **Test UDP Connectivity**
```bash
# From VM1 to VM2
nc -u -l -p 40000 &  # Listen on VM2
echo "test" | nc -u [VM2_IP] 40000  # Send from VM1
```

### **Alternative: Direct UDP Mode**

If WebRTC doesn't work in VMs, use the direct UDP tools:

```bash
# On receiving VM
cargo run --bin udp-receiver

# On sending VM
cargo run --bin udp-test [RECEIVER_IP]
```

### **Performance Tips**
- Use **bridged networking** for best performance
- Close unnecessary VM applications
- Ensure adequate RAM (2GB+ per VM)
- Use SSD storage for better I/O