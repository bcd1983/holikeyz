# Enhanced Elgato Ring Light Controller

## Overview

The enhanced backend provides comprehensive functionality for discovering, connecting to, and controlling Elgato Ring Light devices. It includes advanced features like credential management using the OS keyring, batch provisioning, and interactive control modes.

## Key Features

### 1. Device Discovery & Management
- **Automatic scanning** for Elgato devices in setup mode
- **Detection of device types** (Ring Light, Key Light, Key Light Air)
- **Signal strength indicators** for optimal device selection
- **Support for multiple simultaneous devices**

### 2. Secure Credential Storage
- **OS keyring integration** for secure WiFi password storage
- **Automatic credential retrieval** for known networks
- **Import existing networks** from NetworkManager
- **Encrypted storage** of sensitive information

### 3. Advanced Control Features
- **Scene presets**: daylight, reading, video, relax, warm, cool, focus, evening
- **Animation effects**: fade in/out, pulse, rainbow
- **Batch operations** for multiple devices
- **Interactive control mode** with real-time adjustments

### 4. WiFi Provisioning
- **Automatic network restoration** after provisioning
- **Support for various security types** (Open, WEP, WPA/WPA2/WPA3)
- **Device verification** after provisioning
- **Batch provisioning** for multiple devices

## Installation

```bash
cd holikeyz-ring-light-controller
cargo build --release
sudo cp target/release/elgato-enhanced /usr/local/bin/
```

## Usage Examples

### 1. Discover Devices

```bash
# Scan for devices in setup mode
elgato-enhanced discover

# Include configured devices on the network
elgato-enhanced discover --all

# Output in JSON format
elgato-enhanced discover --format json
```

### 2. Interactive Setup Wizard

```bash
# Interactive setup with prompts
elgato-enhanced setup

# Auto-connect to strongest signal device
elgato-enhanced setup --auto

# Specify target network and use saved credentials
elgato-enhanced setup --network "MyHomeWiFi" --use-saved
```

### 3. Control Devices

```bash
# Turn light on
elgato-enhanced control on

# Set brightness to 75%
elgato-enhanced control brightness 75

# Apply a scene
elgato-enhanced control scene video

# Interactive control mode
elgato-enhanced control interactive

# Animation effects
elgato-enhanced control fade-in
elgato-enhanced control pulse
elgato-enhanced control rainbow
```

### 4. Batch Operations

```bash
# Provision multiple devices to the same network
elgato-enhanced batch "MyHomeWiFi"

# Only provision devices in setup mode
elgato-enhanced batch "MyHomeWiFi" --setup-only
```

### 5. Quick Actions

```bash
# Turn all lights on/off
elgato-enhanced quick all-on
elgato-enhanced quick all-off

# Apply scene to all devices
elgato-enhanced quick all-scene relax

# Flash all lights (notification)
elgato-enhanced quick flash --times 5
```

### 6. Credential Management

```bash
# List saved networks
elgato-enhanced credentials list

# Add network credentials
elgato-enhanced credentials add "MyNetwork" --password "secret"

# Import from NetworkManager
elgato-enhanced credentials import

# Remove saved network
elgato-enhanced credentials remove "OldNetwork"

# Clear all credentials
elgato-enhanced credentials clear
```

### 7. Device Profiles

```bash
# Create a device profile
elgato-enhanced profiles create "ABC123" "Office Light"

# List all profiles
elgato-enhanced profiles list

# Apply a profile
elgato-enhanced profiles apply "Office Light"

# Delete a profile
elgato-enhanced profiles delete "Old Profile"
```

## Architecture

### Core Components

1. **EnhancedProvisioningManager**: Main orchestrator for all device operations
2. **WiFiManager**: Handles network scanning and connections using NetworkManager
3. **CredentialManager**: Secure storage using system keyring
4. **ElgatoProvisioner**: Device-specific provisioning logic
5. **Discovery Module**: mDNS-based device discovery

### Security Features

- Credentials stored in OS keyring (not plain text)
- Automatic cleanup of temporary connections
- Secure WiFi provisioning with encryption
- Network isolation during setup mode

### Command Flow

1. **Discovery Phase**
   - Scan WiFi networks for Elgato SSIDs
   - Use mDNS to find configured devices
   - Sort by signal strength

2. **Connection Phase**
   - Save current network connection
   - Connect to device's setup AP
   - Verify device accessibility

3. **Provisioning Phase**
   - Retrieve/prompt for credentials
   - Send encrypted configuration
   - Device reboots and joins network

4. **Restoration Phase**
   - Restore original network
   - Verify device on new network
   - Update device registry

## Advanced Features

### Scene Definitions

| Scene | Brightness | Temperature | Use Case |
|-------|------------|-------------|----------|
| daylight | 100% | 7000K | Bright daylight |
| reading | 80% | 5000K | Reading/studying |
| video | 90% | 4500K | Video calls |
| relax | 60% | 3500K | Relaxation |
| warm | 70% | 2900K | Warm ambiance |
| cool | 85% | 7000K | Cool white |
| focus | 100% | 5500K | Concentration |
| evening | 50% | 3100K | Evening mood |

### Network Requirements

- WiFi adapter with AP scanning capability
- NetworkManager for connection management
- sudo privileges for network operations

### Supported Devices

- Elgato Ring Light
- Elgato Key Light
- Elgato Key Light Air
- Other Elgato lighting products with WiFi

## Troubleshooting

### Device Not Found
1. Ensure device is in setup mode (hold button 10-15 seconds)
2. Check WiFi adapter is enabled
3. Move closer to the device for better signal

### Provisioning Fails
1. Verify WiFi credentials are correct
2. Ensure target network is 2.4GHz (some devices don't support 5GHz)
3. Check firewall settings

### Connection Issues
1. Restart NetworkManager: `sudo systemctl restart NetworkManager`
2. Clear saved connections: `nmcli connection delete <connection-name>`
3. Check device firmware is up to date

## API Integration

The enhanced manager can be integrated into other applications:

```rust
use holikeyz::provisioning::enhanced_manager::EnhancedProvisioningManager;

async fn example() {
    let manager = EnhancedProvisioningManager::new(None)?;
    
    // Discover devices
    let devices = manager.scan_for_devices(false).await?;
    
    // Connect and control
    for device in devices {
        let connected = manager.connect_to_device(&device).await?;
        
        // Send commands
        let command = LightCommand {
            command_type: CommandType::SetScene,
            parameters: hashmap!{"scene" => "video"},
        };
        manager.send_command(&device.ssid, command).await?;
    }
}
```

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust best practices
- New features include documentation
- Security considerations are addressed
- Tests are included for new functionality

## License

MIT License - See LICENSE file for details