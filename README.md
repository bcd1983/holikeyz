# Holikeyz Ring Light Controller

An unofficial, open-source Ring Light controller for Linux with GNOME integration. Control your ring light with a beautiful Philips Hue-inspired interface directly from your GNOME desktop.

## Legal & Reverse-Engineering Notice

This is an **independent, unofficial** project. It is **not affiliated with, endorsed by, sponsored by, or associated with Elgato, Corsair, or any of their subsidiaries**. All product names, trademarks, and registered trademarks are property of their respective owners; references to "Elgato" are used solely to describe device compatibility.

The device-side protocol constants and wire format used in `src/provisioning/` were derived by black-box interoperability analysis of a device the author lawfully owns, for the purpose of enabling that device to operate with non-vendor software. No vendor firmware, SDKs, or proprietary source code were decompiled, redistributed, or used in producing this project.

This project is provided **"AS IS"**, under the MIT license, for personal, educational, and interoperability use. Use on networks or devices you do not own or have explicit permission to operate may violate local law — you are responsible for how you use it.

## Features

- 🎨 Beautiful GNOME Shell extension with scene presets
- 🎯 Ultra-low latency control (~50ms response time)
- 🖼️ AI-generated scene thumbnails for visual selection
- Complete control of all Ring Light settings
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
git clone https://github.com/yourusername/holikeyz
cd holikeyz

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
export RING_LIGHT_IP=192.168.7.80
export RING_LIGHT_PORT=9123
```

Or modify `~/.config/systemd/user/holikeyz-ring-light.service`

## Usage

### Command Line Interface

```bash
# Turn light on/off
holikeyz-cli on
holikeyz-cli off
holikeyz-cli toggle

# Adjust brightness (0-100)
holikeyz-cli brightness 75

# Set color temperature (2900-7000K)
holikeyz-cli temperature 5600

# Apply scene presets
holikeyz-cli scene daylight
holikeyz-cli scene warm
holikeyz-cli scene video

# Get current status
holikeyz-cli status

# Discover lights on network
holikeyz-cli discover

# Make light flash for identification
holikeyz-cli identify
```

### D-Bus Service

Enable and start the systemd service:

```bash
make enable-service
```

The service exposes the following D-Bus interface:
- `com.holikeyz.RingLight` at `/com/holikeyz/RingLight`

### GNOME Shell Extension

1. Install the extension:
   ```bash
   make install-extension
   ```

2. Restart GNOME Shell (Alt+F2, type 'r', press Enter)

3. Enable the extension using GNOME Extensions app or:
   ```bash
   gnome-extensions enable holikeyz-ring-light@example.com
   ```

The extension provides:
- Panel indicator showing light status
- Quick toggle switch
- Brightness and temperature sliders
- Scene presets menu
- Light identification feature

## API Endpoints

The Ring Light device exposes a REST API on port 9123:

- `GET/PUT /elgato/lights` - Light state control
- `GET /elgato/accessory-info` - Device information
- `GET/PUT /elgato/settings` - Device settings
- `POST /elgato/identify` - Flash the light

## Development

### Project Structure

```
holikeyz/
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
holikeyz-cli --ip 192.168.1.100 status
```

## Temperature Conversion

The API uses internal values (143-344) for color temperature.
The library automatically converts between Kelvin (2900-7000K) and API values.

## Troubleshooting

1. **Light not found**: Ensure the light is on the same network and the IP is correct
2. **D-Bus service fails**: Check systemd logs: `journalctl --user -u holikeyz-ring-light`
3. **Extension not showing**: Restart GNOME Shell and check extension is enabled

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Pull requests are welcome! Please ensure:
- Code follows Rust best practices
- Tests pass
- Documentation is updated

## Acknowledgments

- Thanks to the Rust community for excellent async libraries
- GNOME team for the extensible Shell architecture
- AI image generation powered by Flux Schnell model

## Disclaimer

This is an unofficial, community-driven project. The developers of this software are not affiliated with Elgato, Corsair, or any other hardware manufacturer. Use at your own risk.