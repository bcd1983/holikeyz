# Holikeyz (Elgato Ring Light Controller for Linux)

An unofficial, open-source Ring Light controller for Linux, with native panel widgets for **KDE Plasma** and **GNOME Shell**. Talks to your light directly over the local network — no cloud, no vendor app required.

![architecture](https://img.shields.io/badge/status-personal%20project-blue) ![license](https://img.shields.io/badge/license-MIT-green)

## Legal & Reverse-Engineering Notice

This is an **independent, unofficial** project. It is **not affiliated with, endorsed by, sponsored by, or associated with Elgato, Corsair, or any of their subsidiaries**. All product names, trademarks, and registered trademarks are property of their respective owners; references to "Elgato" are used solely to describe device compatibility.

The device-side protocol constants and wire format used in `src/provisioning/` were derived by black-box interoperability analysis of a device the author lawfully owns, for the purpose of enabling that device to operate with non-vendor software. No vendor firmware, SDKs, or proprietary source code were decompiled, redistributed, or used in producing this project.

This project is provided **"AS IS"**, under the MIT license, for personal, educational, and interoperability use.

## Features

- **KDE Plasma 6 panel widget** with iOS-Philips-Hue-inspired UI, scene thumbnails, and in-popup light discovery.
- **GNOME Shell extension** with panel indicator, sliders, and scene presets.
- **Single D-Bus service** (`com.holikeyz.RingLight`) as the source of truth — any desktop frontend uses the same backend.
- **Network discovery via mDNS** — find lights automatically.
- **Command-line interface** for scripting / automation.
- **Switchable active light**: pick from discovered lights at runtime; choice persists across service restarts.
- **Scene presets**: Daylight, Warm, Cool, Reading, Video, Relax.
- **Ultra-low-latency** HTTP client (~50ms toggle response).

## Architecture

```
┌─────────────────────────┐     ┌─────────────────────────┐
│  KDE plasmoid   /       │     │  holikeyz-cli           │
│  GNOME extension        │     │  (terminal clients)     │
└───────────┬─────────────┘     └────────────┬────────────┘
            │        D-Bus (com.holikeyz.RingLight)        │
            └──────────────┬────────────────┘
                           ▼
            ┌──────────────────────────────┐
            │  holikeyz-service            │
            │  (Rust daemon, session bus)  │
            │  - state cache               │
            │  - mDNS discovery            │
            │  - active-light persistence  │
            └──────────────┬───────────────┘
                           │ HTTP / Elgato-compat API
                           ▼
                    ┌──────────────┐
                    │  Ring Light  │
                    │  (on LAN)    │
                    └──────────────┘
```

The daemon is session-bus auto-activated — calling any method on `com.holikeyz.RingLight` starts it on demand; it shuts down with your session.

## Prerequisites

- Rust 1.70+ and Cargo
- KDE Plasma 6 (for the plasmoid) *or* GNOME Shell 45+ (for the extension)
- `qdbus6` (ships with KDE/Qt 6) if you're using the plasmoid
- `dbus-daemon` (any Linux desktop has it)

## Install

### Build once

```bash
git clone https://github.com/bcd1983/holikeyz-ring-light-controller
cd holikeyz-ring-light-controller
cargo build --release
```

That produces `target/release/holikeyz-cli`, `target/release/holikeyz-service`, and the provisioner binaries.

### Install the D-Bus service

Pick one: **user-level** (no sudo, recommended for a single-user desktop) or **system-wide**.

#### User-level (no sudo)

```bash
# 1. Drop the binary into ~/.local/bin (make sure it's in your PATH).
install -Dm755 target/release/holikeyz-service ~/.local/bin/holikeyz-service

# 2. Register the session-bus service file so D-Bus auto-activates it.
mkdir -p ~/.local/share/dbus-1/services
cat > ~/.local/share/dbus-1/services/com.holikeyz.RingLight.service <<EOF
[D-BUS Service]
Name=com.holikeyz.RingLight
Exec=$HOME/.local/bin/holikeyz-service
EOF

# 3. Tell the running session bus to rescan.
dbus-send --session --type=method_call --dest=org.freedesktop.DBus \
          /org/freedesktop/DBus org.freedesktop.DBus.ReloadConfig
```

The first D-Bus call (e.g., opening the plasmoid popup) will launch the service.

#### System-wide (sudo)

Use the bundled installer — it builds, installs binaries to `/usr/local/bin`, writes the D-Bus and systemd service files, and prompts for your light's IP:

```bash
./install.sh
```

Or via `make`:

```bash
make build
sudo make install       # binaries + /usr/share/dbus-1/services entry
make enable-service     # optional: systemd user unit for always-on
```

### Install the desktop frontend

#### KDE Plasma 6 plasmoid

```bash
cd kde-plasmoid
./install.sh            # runs kpackagetool6 -i or -u
```

Then: right-click your panel → **Add or Manage Widgets** → search "**Ring Light**" → drag it in.

If the widget doesn't appear in the list, reload Plasma Shell:

```bash
kquitapp6 plasmashell && kstart plasmashell
```

#### GNOME Shell extension

From repo root:

```bash
make install-extension
gnome-extensions enable holikeyz-ring-light@example.com
```

Then restart GNOME Shell: Alt+F2 → type `r` → Enter (X11 only). On Wayland, log out and back in.

## Usage

### CLI

The CLI talks directly to the light's HTTP API — it does *not* go through the D-Bus service, so it works with or without the service running.

```bash
# --ip is required unless your light is at the default (192.168.7.80)
holikeyz-cli --ip 192.168.6.80 discover
holikeyz-cli --ip 192.168.6.80 status
holikeyz-cli --ip 192.168.6.80 on
holikeyz-cli --ip 192.168.6.80 off
holikeyz-cli --ip 192.168.6.80 toggle
holikeyz-cli --ip 192.168.6.80 brightness 75
holikeyz-cli --ip 192.168.6.80 temperature 5600
holikeyz-cli --ip 192.168.6.80 scene daylight    # daylight|warm|cool|reading|video|relax
holikeyz-cli --ip 192.168.6.80 identify          # flash the light
```

### D-Bus (for GUI clients / scripting)

The service exposes `com.holikeyz.RingLight` on the session bus at `/com/holikeyz/RingLight`, interface `com.holikeyz.RingLight.Control`.

```bash
# Discover lights on the network (returns a JSON string)
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.Discover 4

# What's the currently active light?
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.GetActiveLight

# Switch the active light
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.SetActiveLight 192.168.6.80 9123

# State queries / mutations
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.GetStatus            # (bool, u8, u32)
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.TurnOn
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.SetBrightness 75
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.SetTemperature 5600
qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight \
       com.holikeyz.RingLight.Control.ApplyScene daylight
```

Full method list is in `src/bin/dbus_service.rs` (`register_interface`).

### Configuring the active light

The D-Bus service persists the active light to `~/.config/holikeyz/active.json`:

```json
{
  "ip": "192.168.6.80",
  "port": 9123
}
```

Priority on startup: `active.json` → `RING_LIGHT_IP`/`RING_LIGHT_PORT` env vars → default (`192.168.7.80:9123`). The plasmoid's "Discover" button calls `SetActiveLight` which rewrites this file.

### systemd (optional)

If you want the service running constantly rather than D-Bus-activated on demand:

```bash
# User unit (recommended)
cp systemd/holikeyz-ring-light.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now holikeyz-ring-light.service
journalctl --user -u holikeyz-ring-light.service -f
```

## API Reference (device)

The Ring Light device exposes a REST API on port 9123 (Elgato-compatible):

| Method | Path | Purpose |
|--------|------|---------|
| `GET` / `PUT` | `/elgato/lights` | Read / set on-off, brightness, temperature |
| `GET` | `/elgato/accessory-info` | Model, firmware, serial |
| `GET` / `PUT` | `/elgato/settings` | Power-on behavior, transition timings |
| `POST` | `/elgato/identify` | Flash the light |

## Project Structure

```
holikeyz-ring-light-controller/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── api.rs              # HTTP client for the device
│   ├── discovery.rs        # mDNS discovery
│   ├── models.rs           # Data structures + temperature conversion
│   ├── error.rs            # Error types
│   ├── provisioning/       # Soft-AP onboarding for new-out-of-box devices
│   └── bin/
│       ├── cli.rs                  # holikeyz-cli
│       ├── dbus_service.rs         # holikeyz-service (desktop backend)
│       ├── provisioning_service.rs # holikeyz-provisioning (local HTTP)
│       ├── elgato_provisioner.rs   # elgato-provisioner
│       └── elgato_enhanced.rs      # elgato-enhanced
├── kde-plasmoid/           # KDE Plasma 6 widget (QML)
├── gnome-extension/        # GNOME Shell extension (JS)
├── systemd/                # Optional systemd user unit
├── dbus/                   # System-bus activation file (used by ./install.sh)
├── examples/               # Example clients
├── install.sh              # One-shot installer (sudo, system-wide)
├── Makefile                # Build / install targets
└── Cargo.toml
```

## Temperature Conversion

The device's API uses internal mired-like values (143–344) for color temperature. The library converts to/from Kelvin (2900–7000K) automatically; all public APIs (CLI, D-Bus, HTTP client) speak Kelvin.

## Troubleshooting

**Plasmoid says "Service offline"**
- Check the service starts: `qdbus6 com.holikeyz.RingLight /com/holikeyz/RingLight com.holikeyz.RingLight.Control.GetActiveLight`
- If you installed user-level, make sure `~/.local/bin/holikeyz-service` exists and is executable.
- Run the binary directly to see logs: `RUST_LOG=info ~/.local/bin/holikeyz-service`

**Plasmoid shows "No light selected"**
- Click the wi-fi icon in the popup header → pick a light from the discovery list. Takes ~3 seconds.

**"Discover" returns no lights**
- Make sure the light is powered on and on the same Wi-Fi (mDNS doesn't cross VLANs / subnets).
- Verify from the CLI: `holikeyz-cli discover` (bypasses the service).
- Some Wi-Fi routers block mDNS / Bonjour by default — check "mDNS proxy" or "Bonjour" settings.

**GNOME extension not showing**
- On Wayland, a full log-out is required after enabling. Alt+F2 → `r` only works on X11.
- Check journalctl: `journalctl --user -f /usr/bin/gnome-shell | grep -i holikeyz`

**D-Bus service running in the wrong place**
- Session bus activation tries system (`/usr/share/dbus-1/services/`) before user (`~/.local/share/dbus-1/services/`). If you have *both*, the system entry wins. Either remove the system one or make sure it points to the binary you want.

## License

MIT — see [LICENSE](LICENSE).
