# openlinkhub-tray

A system tray icon for [OpenLinkHub](https://github.com/jurkovic-nikola/OpenLinkHub), written in Rust.

It talks to the desktop over D-Bus using the StatusNotifierItem protocol (via [ksni](https://crates.io/crates/ksni)), and to OpenLinkHub over its local HTTP API.

## Features

- Lists the user profiles for each OpenLinkHub device and lets you switch between them. The active profile is marked.
  - With several devices, each device gets its own submenu.
  - With one device, its profiles are shown directly in the menu.
- **Open**, or left-clicking the icon, opens the OpenLinkHub dashboard in your default browser (`xdg-open`).
- **Refresh** reloads the profiles. They are also refreshed on a timer and whenever the menu opens.
- The tooltip shows the active profile for each device.
- **Quit** exits the tray.

## Requirements

- A running OpenLinkHub service (default `127.0.0.1:27003`)
- A desktop or bar with StatusNotifierItem support (KDE Plasma, Waybar, GNOME with the AppIndicator extension, etc.)
- `xdg-open` (from `xdg-utils`)
- Rust toolchain (edition 2024) to build

## Build

```sh
cargo build --release
```

The binary is written to `target/release/openlinkhub-tray`.

## Install

```sh
cargo install --path .
```

This installs `openlinkhub-tray` to `~/.cargo/bin`.

### Autostart

Hyprland (with uwsm):

```ini
exec-once = uwsm app -- ~/.cargo/bin/openlinkhub-tray
```

Or as a systemd user service, saved as `~/.config/systemd/user/openlinkhub-tray.service`:

```ini
[Unit]
Description=OpenLinkHub tray icon
PartOf=graphical-session.target
After=graphical-session.target

[Service]
ExecStart=%h/.cargo/bin/openlinkhub-tray
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

```sh
systemctl --user daemon-reload
systemctl --user enable --now openlinkhub-tray.service
```

## Usage

```
openlinkhub-tray [--ip <IP>] [--port <PORT>] [--interval <SECONDS>]
```

| Option       | Default     | Description                              |
|--------------|-------------|------------------------------------------|
| `--ip`       | `127.0.0.1` | Host or IP address of OpenLinkHub        |
| `--port`     | `27003`     | Port of OpenLinkHub                      |
| `--interval` | `30`        | Seconds between automatic profile refreshes |

## OpenLinkHub API used

- `GET /api/devices/`: lists devices and their `userProfiles`
- `POST /api/userProfile/change`: switches a profile, with the body `{"deviceId": "...", "userProfileName": "..."}`
