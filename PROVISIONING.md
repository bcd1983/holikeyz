# Soft AP Provisioning for Elgato Ring Lights

This module implements the soft AP provisioning flow used by Elgato and other IoT devices to connect them to WiFi networks during initial setup.

## Overview

The provisioning flow allows your backend to:
- Discover devices in setup mode
- Connect to their temporary WiFi access point
- Send WiFi credentials securely
- Monitor the connection process
- Verify successful network integration

## Architecture

### Components

1. **Soft AP Manager** (`soft_ap.rs`)
   - Manages WiFi Direct/Soft AP creation
   - Handles hostapd and dnsmasq configuration
   - Provides DHCP services for connected devices

2. **WiFi Manager** (`wifi_manager.rs`)
   - Scans for available networks
   - Manages network connections via NetworkManager
   - Verifies internet connectivity

3. **Device Provisioner** (`device_provisioner.rs`)
   - Elgato-specific provisioning protocol
   - Generic provisioning for other devices
   - Handles credential exchange and verification

4. **Security Manager** (`security.rs`)
   - Session token management
   - Credential encryption
   - Device authentication

5. **Provisioning API** (`api.rs`)
   - RESTful endpoints for provisioning workflow
   - Session management
   - Real-time status updates

## Elgato Provisioning Flow

### 1. Device Setup Mode
```bash
# Put Elgato Ring Light in setup mode
# Hold the button on the back for 10 seconds until it blinks
# Device creates WiFi AP: "Elgato Ring Light XXXX"
```

### 2. Connect to Device AP
```rust
// The device creates its own soft AP
// Connect your system to this network
let device_ssid = "Elgato Ring Light 1A2B";
```

### 3. Send WiFi Credentials
```rust
// POST to http://192.168.4.1:9123/elgato/wifi/update
{
    "ssid": "YourHomeWiFi",
    "pass": "YourPassword",
    "security": 3,  // WPA2
    "priority": 1
}
```

### 4. Verify Connection
The device will:
- Disconnect its soft AP
- Connect to the target network
- Become discoverable via mDNS

## Usage

### Command Line Interface

```bash
# Start the provisioning server
cargo run --bin holikeyz-provisioning -- server --port 9124

# Scan for WiFi networks
cargo run --bin holikeyz-provisioning -- scan

# Provision an Elgato device
cargo run --bin holikeyz-provisioning -- provision \
    --device-type elgato \
    --device-ssid "Elgato Ring Light 1A2B" \
    --target-ssid "YourWiFi" \
    --target-password "YourPassword"

# Discover devices on network
cargo run --bin holikeyz-provisioning -- discover --timeout 30
```

### API Endpoints

#### Start Provisioning Session
```http
POST /provisioning/start
{
    "device_type": "elgato",
    "device_ssid": "Elgato Ring Light 1A2B"
}
```

#### Scan Networks
```http
GET /provisioning/scan
```

#### Send Credentials
```http
POST /provisioning/provision
{
    "session_id": "uuid",
    "wifi_credentials": {
        "ssid": "YourWiFi",
        "password": "YourPassword",
        "security_type": "WPA2",
        "hidden": false
    },
    "device_name": "Living Room Light",
    "timezone": "America/New_York",
    "locale": "en_US"
}
```

#### Check Status
```http
GET /provisioning/status/{session_id}
```

#### Stop Provisioning
```http
POST /provisioning/stop/{session_id}
```

### Example Client

Run the interactive example:
```bash
cargo run --example provisioning_client
```

This will guide you through:
1. Starting a provisioning session
2. Connecting to the device's WiFi
3. Scanning for networks
4. Sending credentials
5. Monitoring the connection
6. Discovering the device on your network

## Requirements

### System Requirements
- Linux with NetworkManager
- WiFi adapter with AP mode support
- hostapd and dnsmasq (for creating soft AP)
- Root/sudo access for network configuration

### Check WiFi Capabilities
```bash
# Check if your WiFi adapter supports AP mode
iw list | grep -A 5 "Supported interface modes"
```

## Security Considerations

1. **Credential Protection**
   - Credentials are encrypted during transmission
   - Session tokens expire after 1 hour
   - HMAC signatures verify data integrity

2. **Network Isolation**
   - Provisioning happens on isolated network
   - Limited time window for provisioning
   - Device verification before credential exchange

3. **Best Practices**
   - Always use WPA2/WPA3 for target networks
   - Implement rate limiting on API endpoints
   - Log all provisioning attempts
   - Rotate security keys periodically

## Troubleshooting

### Device Not Found
- Ensure device is in setup mode
- Check WiFi adapter is working
- Verify mDNS service is running

### Connection Failed
- Verify WiFi credentials are correct
- Check signal strength
- Ensure network allows new devices

### Permission Errors
- Run with appropriate permissions for network configuration
- Check NetworkManager policies
- Verify hostapd/dnsmasq installation

## Testing

```bash
# Run unit tests
cargo test --lib provisioning

# Integration test with mock device
cargo test --test provisioning_integration

# Manual testing with real device
1. Put Elgato Ring Light in setup mode
2. Run provisioning server
3. Use example client or curl commands
4. Verify device appears on network
```

## Future Enhancements

- [ ] Bluetooth provisioning support
- [ ] QR code based setup
- [ ] Enterprise network support (802.1x)
- [ ] Multi-device batch provisioning
- [ ] Cloud-based provisioning backup
- [ ] Mobile app integration
- [ ] Automatic firmware updates during provisioning