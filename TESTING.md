# VoIP Application - Testing Guide

## 🧪 Testing Overview

This guide covers manual testing procedures, test cases, and debugging tips.

## ✅ Pre-Test Checklist

Before testing:
- [ ] Rust installed (`rustc --version`)
- [ ] Python installed (`python --version`)
- [ ] Both backend and frontend running
- [ ] No other apps using ports 8080 or 3000
- [ ] Microphone connected and working
- [ ] Browser console open (F12)
- [ ] Network tab visible in DevTools

## 🚀 Test Scenarios

### Scenario 1: User Registration

**Steps**:
1. Open `http://localhost:3000` in browser
2. A prompt appears asking for username
3. Enter a username (e.g., "Alice")
4. Click OK

**Expected Result**:
- ✅ Page loads fully
- ✅ Header shows "Connected as: Alice"
- ✅ Status badge shows "IDLE"
- ✅ User list starts loading

**Debugging**:
- Console error? Check backend running
- "Not Connected"? Check API_BASE in app.js

### Scenario 2: User List Display

**Steps**:
1. Complete Scenario 1 (register as Alice)
2. Open new browser window/tab
3. Register as different user (e.g., "Bob")
4. In Alice's window, check user list

**Expected Result**:
- ✅ Bob appears in left panel
- ✅ Bob has status indicator (grey dot = IDLE)
- ✅ List updates every 5 seconds
- ✅ Clicking Bob's name should select them

**Debugging**:
- No users appear? Network tab → check API call to `/users/list`
- Status not updating? Check setInterval in app.js line 44

### Scenario 3: Call Initiation (Direct User)

**Steps**:
1. Have 2 users open (Alice and Bob)
2. In Alice's window, select "Bob" from dropdown
3. Click "Call" button
4. Observe Bob's window

**Expected Result**:
- ✅ Alice's status changes to "CALLING" (yellow, pulsing)
- ✅ Modal appears in Bob's window: "Call from: Alice"
- ✅ Call controls appear in Alice's window
- ✅ Alice sees "Current Call Info: Bob"

**Debugging**:
- Modal doesn't appear? 
  - Check browser console for errors
  - Verify simulateIncomingCall() called (line 388)
- Status not changing?
  - Check updateStatus() in app.js
  - Verify API response in Network tab

### Scenario 4: Call Acceptance

**Steps**:
1. From Scenario 3, click "Accept" in Bob's modal
2. Observe both windows

**Expected Result**:
- ✅ Modal closes in Bob's window
- ✅ Both show status "IN CALL" (green, pulsing)
- ✅ Call timer starts (00:00 → 00:01 → 00:02...)
- ✅ Both see call controls (Mute/Hold/End)
- ✅ Audio visualization starts animating

**Debugging**:
- Status doesn't change?
  - Check `/api/signal/accept` response in Network tab
  - Check appState.callStartTime set correctly
- Timer doesn't start?
  - Check updateCallTimer() function
  - Verify setInterval at line 46

### Scenario 5: Call Rejection

**Steps**:
1. Start new call (Alice to Bob)
2. In Bob's modal, click "Reject"
3. Observe both windows

**Expected Result**:
- ✅ Modal closes in Bob's window
- ✅ Call controls hidden in Alice's window
- ✅ Both return to "IDLE" status
- ✅ Both can initiate new calls

**Debugging**:
- Status doesn't return to IDLE?
  - Check endCallCleanup() function
  - Verify resetStatus() called

### Scenario 6: Mute Toggle

**Steps**:
1. Establish active call (Scenario 4)
2. Click "Mute" button
3. Check button changes
4. Click "Unmute" button

**Expected Result**:
- ✅ Button text changes to "Unmute" (🔇)
- ✅ Button gets highlighted/active state
- ✅ Microphone audio is disabled
- ✅ Toggle back to unmute (🔊)

**Debugging**:
- Button doesn't change?
  - Check toggleMute() function
  - Verify button ID correct
- Audio still transmitting?
  - Check track.enabled = !appState.isMuted
  - Verify localStream exists

### Scenario 7: Hold/Resume

**Steps**:
1. Establish active call
2. Click "Hold" button
3. Status should change to "ON HOLD"
4. Click "Resume" button
5. Status returns to "IN CALL"

**Expected Result**:
- ✅ Button text changes to "Resume" (⏯)
- ✅ Status badge changes to "ON HOLD" (orange)
- ✅ Click "Resume", status returns to "IN CALL"
- ✅ Button text changes back to "Hold" (⏸)
- ✅ Both parties show same status

**Debugging**:
- Status not changing?
  - Check holdCall() function
  - Verify `/api/signal/hold` endpoint response
- Doesn't show ON HOLD?
  - Check updateStatus('on-hold') called
  - Verify CSS class for on-hold styling

### Scenario 8: End Call

**Steps**:
1. Establish call in any state (active, on hold)
2. Click "End Call" button
3. Observe both windows

**Expected Result**:
- ✅ Call controls disappear
- ✅ Call info section disappears
- ✅ Both return to "IDLE" status
- ✅ Timer resets
- ✅ Both can make new calls immediately

**Debugging**:
- Controls don't disappear?
  - Check hideCallControls() function
  - Verify element IDs match
- Audio continues?
  - Check stopAllTracks() in endCallCleanup()
  - Verify localStream nullified

### Scenario 9: Call by IP Address

**Steps**:
1. Get your computer's IP (ipconfig or ifconfig)
2. In one browser window, enter IP in "Enter Target IP Address"
3. Click "Call by IP"
4. Check other window for incoming call

**Expected Result**:
- ✅ Call initiated to the IP
- ✅ Status changes to "CALLING"
- ✅ Incoming call modal appears (if simulated)
- ✅ Call can be accepted/rejected

**Debugging**:
- Call doesn't initiate?
  - Check IP format (e.g., 192.168.1.100)
  - Verify API response in Network tab
- Input field issue?
  - Check input field ID: target-ip
  - Verify click handler on button

### Scenario 10: Audio Visualization

**Steps**:
1. Establish active call
2. Speak into microphone
3. Observe audio level meter
4. Observe frequency spectrum canvas
5. Watch audio level change with voice

**Expected Result**:
- ✅ Audio level meter shows green bar (increases with sound)
- ✅ Spectrum visualizer shows animated bars
- ✅ Status shows "Active" when sound detected
- ✅ Meters responsive to mic input
- ✅ Colors change from green → yellow → red with volume

**Debugging**:
- Meters not moving?
  - Check setupAudioAnalyzer() called
  - Verify analyser.getByteFrequencyData() works
  - Check browser console for Web Audio API errors
- Canvas shows nothing?
  - Check drawAudioVisualization() function
  - Verify canvas context obtained correctly
  - Check canvas dimensions set properly

### Scenario 11: Responsive Design

**Steps**:
1. Open application in browser
2. Resize window to tablet width (700px)
3. Resize to mobile width (400px)
4. Check layout at each breakpoint

**Expected Result**:
- ✅ Desktop (>1200px): 3-column layout
- ✅ Tablet (700-1200px): 2-column layout
- ✅ Mobile (<700px): 1-column stacked layout
- ✅ All buttons remain functional
- ✅ Text remains readable
- ✅ No horizontal scrolling

**Debugging**:
- Elements overlapping?
  - Check media queries in styles.css
  - Verify grid-template-columns
- Buttons too small?
  - Check padding in mobile breakpoint
  - Verify touch targets ≥44px

## 🔍 Detailed Test Cases

### Test Case 1.1: Register User

```
Given: Application loaded
When: User enters username "TestUser"
Then: User registered with unique ID
And: Status shows "Connected as: TestUser"
And: Status badge shows "IDLE"
```

### Test Case 2.1: View User Status

```
Given: Two users registered (Alice, Bob)
When: Alice opens app
Then: Bob appears in user list
And: Bob has status indicator
And: Status indicator matches Bob's state
```

### Test Case 3.1: Initiate Call to Online User

```
Given: Alice and Bob both online
When: Alice selects Bob and clicks "Call"
Then: Alice status changes to "CALLING"
And: Modal appears in Bob's window
And: Call ID generated and stored
```

### Test Case 4.1: Accept Incoming Call

```
Given: Incoming call modal shown
When: User clicks "Accept" button
Then: Modal closes
And: Both users status becomes "IN CALL"
And: Call timer starts
And: Call controls appear
```

### Test Case 4.2: Reject Incoming Call

```
Given: Incoming call modal shown
When: User clicks "Reject" button
Then: Modal closes
And: Both users status returns to "IDLE"
And: Call is terminated server-side
```

### Test Case 5.1: Mute During Call

```
Given: Active call in progress
When: User clicks "Mute" button
Then: Button shows "Unmute" (🔇)
And: Microphone is disabled
And: Button appears active/highlighted
```

### Test Case 5.2: Unmute During Call

```
Given: Call is muted
When: User clicks "Unmute" button
Then: Button shows "Mute" (🔊)
And: Microphone is enabled
And: Button appears normal
```

### Test Case 6.1: Hold Call

```
Given: Active call
When: User clicks "Hold" button
Then: Status changes to "ON HOLD"
And: Button text changes to "Resume"
And: Both parties see ON HOLD status
```

### Test Case 6.2: Resume Held Call

```
Given: Call on hold
When: User clicks "Resume" button
Then: Status changes back to "IN CALL"
And: Button text changes to "Hold"
And: Call continues
```

### Test Case 7.1: End Call

```
Given: Active call in any state
When: User clicks "End Call" button
Then: Call ends immediately
And: Both users return to "IDLE"
And: Call controls disappear
And: Timer stops and resets
```

### Test Case 8.1: Call by IP

```
Given: Valid IP address on network
When: User enters IP and clicks "Call by IP"
Then: Call initiates to that IP
And: Status shows "CALLING"
And: Call can be accepted/rejected normally
```

### Test Case 9.1: Audio Permission

```
Given: First call attempt
When: Call initiated
Then: Microphone permission dialog appears
And: User can Grant/Deny permission
And: If granted: audio capture begins
And: If denied: call proceeds without audio
```

### Test Case 10.1: Audio Visualization

```
Given: Active call with audio enabled
When: User speaks into microphone
Then: Audio level meter animates
And: Frequency spectrum displays bars
And: Visualization is smooth and responsive
```

## 🐛 Debugging Techniques

### Browser Console Errors

**Access console**:
1. Press F12 or Ctrl+Shift+I
2. Click "Console" tab
3. Look for red error messages

**Common errors**:

```
TypeError: Cannot read property 'call' of null
→ User not registered, check initializeUser()

TypeError: Cannot read property 'srcObject' of null
→ Audio element not found, check element IDs

CORS error
→ Backend not running or wrong port
```

### Network Tab Debugging

1. Open DevTools (F12)
2. Click "Network" tab
3. Perform action (call initiation, etc.)
4. Look for API requests
5. Check Status Code (should be 200 for success)
6. Click request to see Response

**Expected responses**:

```
/api/users/register: 200 OK
/api/users/list: 200 OK
/api/signal/initiate: 200 OK {"status":"success","call_id":"..."}
```

### Backend Debugging

**Check server logs**:
```bash
# Terminal running backend
# Look for messages like:
[2026-01-10T10:00:00Z INFO voip_backend] Starting VoIP Server
[2026-01-10T10:00:01Z INFO voip_backend] Call initiated: uuid
```

**Enable debug logging**:
```bash
RUST_LOG=debug cargo run --release
```

### JavaScript Debugging

**Add console logs**:
```javascript
console.log('Debug: Current state:', appState);
console.log('Debug: Call initiated:', data);
```

**Set breakpoints**:
1. Open DevTools (F12)
2. Click "Sources" tab
3. Click line number to set breakpoint
4. Refresh page
5. Execution pauses at breakpoint

## 📊 Performance Testing

### Measure API Response Time

```javascript
// In browser console
const start = performance.now();
fetch('http://localhost:8080/api/users/list')
  .then(r => r.json())
  .then(d => {
    const end = performance.now();
    console.log(`API Response: ${(end-start).toFixed(2)}ms`);
  });
```

Expected: <100ms for local network

### Monitor Memory Usage

1. Open DevTools (F12)
2. Click "Memory" tab
3. Take heap snapshot
4. Perform actions (calls, mute, etc.)
5. Take another snapshot
6. Compare differences

Expected: Stable memory, no leaks

### Check CPU Usage

1. Observe server terminal during calls
2. CPU should remain low (background)
3. No spinning/maxed out cores

## ✨ Regression Testing

After making code changes, test these critical paths:

1. **Registration Flow**
   - Can register new user
   - Can view in user list

2. **Call Flow**
   - Can initiate call
   - Can accept call
   - Can reject call
   - Can end call

3. **Call Controls**
   - Mute works
   - Hold works
   - Resume works

4. **Audio**
   - Permission prompt works
   - Audio captures
   - Visualization works

5. **UI**
   - Status badges update
   - Modals appear/disappear
   - Responsive layout works

## 📋 Test Results Template

```
Test Date: ____________
Tester: ________________
Browser: _______________
OS: ____________________
Backend Version: ________

Test Results:
[ ] User Registration       PASS / FAIL
[ ] User List Display       PASS / FAIL
[ ] Call Initiation         PASS / FAIL
[ ] Call Acceptance         PASS / FAIL
[ ] Call Rejection          PASS / FAIL
[ ] Mute Toggle             PASS / FAIL
[ ] Hold/Resume             PASS / FAIL
[ ] End Call                PASS / FAIL
[ ] Call by IP              PASS / FAIL
[ ] Audio Visualization     PASS / FAIL
[ ] Responsive Design       PASS / FAIL

Issues Found:
1. _________________________
2. _________________________

Notes:
_____________________________
```

---

**Last Updated**: 2026-01-10
