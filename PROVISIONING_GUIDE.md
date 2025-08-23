# Elgato Ring Light Provisioning Guide

## Overview

This guide explains how to provision (configure WiFi settings) for your Elgato Ring Light using the enhanced controller.

## Quick Start

```bash
# Build and run the setup wizard
cargo build --release --bin elgato-enhanced
./target/release/elgato-enhanced setup --use-saved
```

## Provisioning Methods

### 1. Automatic API Provisioning (Primary)

The controller uses encrypted API calls to configure the device:

1. **Put device in setup mode**: Hold the button on the back for 10-15 seconds
2. **Run setup wizard**: `./target/release/elgato-enhanced setup --use-saved`
3. **Enter credentials**: If not saved, you'll be prompted for WiFi password
4. **Automatic configuration**: The device receives encrypted credentials

### 2. Web Interface Fallback (Automatic)

If API provisioning fails (400 Bad Request), the system automatically:

1. **Opens web interface**: Browser opens to `http://192.168.62.1:9123`
2. **Manual configuration**: Select your network and enter password in the web UI
3. **Confirmation**: Press Enter in the terminal when done

### 3. Manual Web Configuration

You can also manually configure via web interface:

1. Connect to the device's WiFi (e.g., "Elgato Ring Light ADD0")
2. Open browser to: `http://192.168.62.1:9123`
3. Click "WiFi Settings"
4. Select your network and enter password
5. Click "Connect"

### 4. Mobile App

Use the official Elgato Control Center app:
- iOS: [App Store](https://apps.apple.com/app/elgato-control-center/id1446846905)
- Android: [Google Play](https://play.google.com/store/apps/details?id=com.elgato.controlcenter)

## Credential Management

### Saving Credentials

Credentials are stored securely in your OS keyring:

```bash
# Add credentials manually
./target/release/elgato-enhanced credentials add "YourNetwork" --password "YourPassword"

# List saved networks
./target/release/elgato-enhanced credentials list

# Import from NetworkManager
./target/release/elgato-enhanced credentials import
```

### Security

- Passwords are **never** stored in plain text
- Uses OS keyring (Keychain on macOS, libsecret on Linux)
- Credentials encrypted with AES-256 when sent to device

## Troubleshooting

### "400 Bad Request" Error

This usually means the encryption format doesn't match the device's expectations. The system will automatically fall back to web interface.

### Device Not Found

1. Ensure device is in setup mode (blinking light)
2. Check WiFi is enabled: `nmcli radio wifi on`
3. Scan manually: `nmcli dev wifi list | grep Elgato`

### Connection Fails After Provisioning

1. Check network is 2.4GHz (some devices don't support 5GHz)
2. Verify password is correct
3. Ensure router allows new devices
4. Check firewall settings

### Web Interface Not Opening

If the browser doesn't open automatically:
1. Manually navigate to: `http://192.168.62.1:9123`
2. Ensure you're connected to the device's WiFi
3. Try a different browser

## Network Requirements

- **2.4GHz WiFi**: Most Elgato lights only support 2.4GHz
- **WPA/WPA2**: Recommended security (WPA3 may not work)
- **DHCP**: Router must provide IP addresses
- **mDNS**: For device discovery after provisioning

## Advanced Usage

### Batch Provisioning

Configure multiple devices to the same network:

```bash
./target/release/elgato-enhanced batch "YourNetwork" --setup-only
```

### Direct Control During Setup

Test the device while connected to its AP:

```bash
./target/release/elgato-enhanced control interactive
```

### Custom Interface

Use a specific WiFi adapter:

```bash
./target/release/elgato-enhanced --interface wlan1 setup
```

## API Details

### Encryption

The Elgato API expects:
1. **AES-256-CBC** encryption
2. **Fixed IV**: `049F6F1149C6F84B1B14913C71E9CDBE`
3. **Dynamic key**: Based on device firmware and hardware version
4. **16-byte random prefix**: Prepended to JSON payload
5. **PKCS7 padding**: To align to 16-byte blocks

### Payload Format

```json
{
  "SSID": "YourNetwork",
  "Passphrase": "YourPassword",
  "SecurityType": "2"
}
```

Security types:
- `"0"`: Open
- `"1"`: WEP
- `"2"`: WPA/WPA2/WPA3

## Safe Provisioning Script

Use the included safe provisioning script for the best experience:

```bash
./provision-safe.sh
```

This script:
- Handles all error cases
- Provides clear feedback
- Falls back gracefully
- Saves credentials securely

## Support

If provisioning continues to fail:
1. Check this guide's troubleshooting section
2. Try the official Elgato Control Center app
3. Contact Elgato support with your device model and firmware version
4. File an issue on the project repository