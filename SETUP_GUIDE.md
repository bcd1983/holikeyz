# Elgato Ring Light Setup Guide

## Current Status
- ❌ Elgato Ring Light WiFi network NOT detected
- Your device needs to be in setup mode to provision it

## Step-by-Step Setup Process

### 1. Put Elgato Ring Light in Setup Mode

**Physical Setup:**
1. Locate the control button on the back of your Elgato Ring Light
2. Press and hold the button for **10-15 seconds**
3. The light will blink/flash to indicate it's entering setup mode
4. Release the button
5. Wait 10-30 seconds for the WiFi network to appear

**Expected Result:**
- A new WiFi network named "Elgato Ring Light ADD0" should appear
- This network is typically open (no password required)

### 2. Verify Setup Mode

Run this command to check if the Elgato WiFi is visible:
```bash
nmcli dev wifi list | grep -i elgato
```

You should see something like:
```
3C:XX:XX:XX:AD:D0  Elgato Ring Light ADD0  Infra  6  54 Mbit/s  90  ▂▄▆█  --
```

### 3. Connect to Elgato WiFi

Once the network appears, connect to it:
```bash
nmcli device wifi connect "Elgato Ring Light ADD0"
```

Or manually through your WiFi settings.

### 4. Test the Connection

After connecting, run:
```bash
./provision-elgato.sh
```

This will find the correct IP and API endpoints.

### 5. Provision the Device

Once connected to the Elgato's WiFi, run:
```bash
cargo run --example provisioning_client
```

Enter your home WiFi credentials when prompted.

## Troubleshooting

### Device Won't Enter Setup Mode
- Try unplugging the device for 10 seconds, then plug it back in
- Hold the button longer (up to 20 seconds)
- The light should blink or change behavior when entering setup mode

### WiFi Network Doesn't Appear
- Wait up to 60 seconds after entering setup mode
- Move closer to the device
- Refresh WiFi list: `nmcli dev wifi rescan`
- Check if your WiFi adapter is enabled: `nmcli radio wifi on`

### Can't Connect to Elgato WiFi
- The network should be open (no password)
- If it asks for a password, try "elgato" or leave blank
- Disable your wired connection temporarily if needed

### API Not Responding
The Elgato API typically runs on:
- IP: 192.168.4.1 (most common)
- Port: 9123 or 80
- Endpoints: /elgato/accessory-info, /elgato/wifi/scan, /elgato/wifi/update

## Current Network Status

Your current setup:
- Connected to: Wired network (192.168.6.x)
- Available WiFi networks: BELL824, BELL825, Internet-of-Thangz
- Elgato network: NOT VISIBLE (device not in setup mode)

## Next Steps

1. Put your Elgato Ring Light in setup mode (hold button 10-15 seconds)
2. Run: `watch -n 1 'nmcli dev wifi list | grep -i elgato'` to monitor for the network
3. Once it appears, connect to it
4. Run the provisioning client