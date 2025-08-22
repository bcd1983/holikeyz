# Elgato Ring Light Controller

A cross-platform Rust application to control Elgato Ring Light devices with GNOME Shell integration.

## Features

- Complete control of all Elgato Ring Light settings
- Command-line interface for scripting and automation
- D-Bus service for system integration
- GNOME Shell extension with panel indicator
- Scene presets (Daylight, Warm, Cool, Reading, Video)
- Real-time brightness and color temperature adjustment
- Device discovery via mDNS
- Cross-platform support (Linux, macOS, Windows)

## Prerequisites

- Rust 1.70+ and Cargo
- For GNOME integration: GNOME Shell 45+
- D-Bus (for Linux desktop integration)

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/yourusername/elgato-ring-light-controller
cd elgato-ring-light-controller

# Build the project
make build

# Install binaries and services
sudo make install

# Install GNOME extension
make install-extension
```

### Configure the Ring Light IP

Edit the systemd service file or set environment variables:

```bash
export ELGATO_IP=192.168.7.80
export ELGATO_PORT=9123
```

Or modify `~/.config/systemd/user/elgato-ring-light.service`

## Usage

### Command Line Interface

```bash
# Turn light on/off
elgato-cli on
elgato-cli off
elgato-cli toggle

# Adjust brightness (0-100)
elgato-cli brightness 75

# Set color temperature (2900-7000K)
elgato-cli temperature 5600

# Apply scene presets
elgato-cli scene daylight
elgato-cli scene warm
elgato-cli scene video

# Get current status
elgato-cli status

# Discover lights on network
elgato-cli discover

# Make light flash for identification
elgato-cli identify
```

### D-Bus Service

Enable and start the systemd service:

```bash
make enable-service
```

The service exposes the following D-Bus interface:
- `com.elgato.RingLight` at `/com/elgato/RingLight`

### GNOME Shell Extension

1. Install the extension:
   ```bash
   make install-extension
   ```

2. Restart GNOME Shell (Alt+F2, type 'r', press Enter)

3. Enable the extension using GNOME Extensions app or:
   ```bash
   gnome-extensions enable elgato-ring-light@example.com
   ```

The extension provides:
- Panel indicator showing light status
- Quick toggle switch
- Brightness and temperature sliders
- Scene presets menu
- Light identification feature

## API Endpoints

The Elgato Ring Light exposes a REST API on port 9123:

- `GET/PUT /elgato/lights` - Light state control
- `GET /elgato/accessory-info` - Device information
- `GET/PUT /elgato/settings` - Device settings
- `POST /elgato/identify` - Flash the light

## Development

### Project Structure

```
elgato-ring-light-controller/
├── src/
│   ├── lib.rs           # Core library
│   ├── api.rs           # HTTP API client
│   ├── models.rs        # Data structures
│   ├── discovery.rs     # mDNS discovery
│   ├── error.rs         # Error handling
│   └── bin/
│       ├── cli.rs       # CLI application
│       └── dbus_service.rs # D-Bus service
├── gnome-extension/     # GNOME Shell extension
├── systemd/            # Systemd service files
├── dbus/              # D-Bus service files
└── Cargo.toml         # Rust dependencies
```

### Testing

```bash
# Run tests
cargo test

# Test with specific IP
elgato-cli --ip 192.168.1.100 status
```

## Temperature Conversion

The API uses internal values (143-344) for color temperature.
The library automatically converts between Kelvin (2900-7000K) and API values.

## Troubleshooting

1. **Light not found**: Ensure the light is on the same network and the IP is correct
2. **D-Bus service fails**: Check systemd logs: `journalctl --user -u elgato-ring-light`
3. **Extension not showing**: Restart GNOME Shell and check extension is enabled

## License

MIT License

## Contributing

Pull requests are welcome! Please ensure:
- Code follows Rust best practices
- Tests pass
- Documentation is updated