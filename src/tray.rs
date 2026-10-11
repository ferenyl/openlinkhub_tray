use std::process::Command as Process;
use std::sync::mpsc::Sender;

use ksni::menu::{MenuItem, RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{OfflineReason, Status, ToolTip, Tray};

use crate::api::DeviceProfiles;

pub enum Command {
    Refresh,
    ChangeProfile { serial: String, profile: String },
    Quit,
}

pub struct OpenLinkHubTray {
    pub dashboard_url: String,
    pub devices: Vec<DeviceProfiles>,
    pub error: Option<String>,
    pub commands: Sender<Command>,
}

impl OpenLinkHubTray {
    fn open_dashboard(&self) {
        if let Err(e) = Process::new("xdg-open").arg(&self.dashboard_url).spawn() {
            eprintln!("Failed to open browser: {e}");
        }
    }

    fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    fn profile_group(&self, index: usize) -> MenuItem<Self> {
        let device = &self.devices[index];
        RadioGroup {
            selected: device.active.unwrap_or(usize::MAX),
            select: Box::new(move |tray: &mut Self, selected| {
                let Some(device) = tray.devices.get_mut(index) else {
                    return;
                };
                let Some(profile) = device.profiles.get(selected).cloned() else {
                    return;
                };
                device.active = Some(selected);
                let serial = device.serial.clone();
                tray.send(Command::ChangeProfile { serial, profile });
            }),
            options: device
                .profiles
                .iter()
                .map(|name| RadioItem {
                    label: name.replace('_', "__"),
                    ..Default::default()
                })
                .collect(),
        }
        .into()
    }

    fn profile_items(&self) -> Vec<MenuItem<Self>> {
        if let Some(error) = &self.error {
            return vec![
                StandardItem {
                    label: format!("OpenLinkHub unavailable: {error}"),
                    enabled: false,
                    icon_name: "dialog-warning".into(),
                    ..Default::default()
                }
                .into(),
            ];
        }

        if self.devices.is_empty() {
            return vec![
                StandardItem {
                    label: "No profiles found".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            ];
        }

        if self.devices.len() == 1 {
            let device = &self.devices[0];
            return vec![
                StandardItem {
                    label: format!("Profiles – {}", device.product),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
                self.profile_group(0),
            ];
        }

        (0..self.devices.len())
            .map(|i| {
                SubMenu {
                    label: self.devices[i].product.replace('_', "__"),
                    submenu: vec![self.profile_group(i)],
                    ..Default::default()
                }
                .into()
            })
            .collect()
    }
}

impl Tray for OpenLinkHubTray {
    fn id(&self) -> String {
        "openlinkhub-tray".into()
    }

    fn title(&self) -> String {
        "OpenLinkHub".into()
    }

    fn icon_name(&self) -> String {
        "input-gaming".into()
    }

    fn status(&self) -> Status {
        Status::Active
    }

    fn tool_tip(&self) -> ToolTip {
        let description = match &self.error {
            Some(error) => format!("Unavailable: {error}"),
            None => self
                .devices
                .iter()
                .map(|d| {
                    let active = d.active.and_then(|i| d.profiles.get(i)).map_or("–", |p| p);
                    format!("{}: {}", d.product, active)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        ToolTip {
            title: "OpenLinkHub".into(),
            description,
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.open_dashboard();
    }

    fn menu_about_to_show(&mut self) {
        self.send(Command::Refresh);
    }

    fn watcher_online(&self) {
        eprintln!("StatusNotifierWatcher online, tray registered");
    }

    fn watcher_offline(&self, reason: OfflineReason) -> bool {
        eprintln!("StatusNotifierWatcher offline, waiting for it: {reason:?}");
        true
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items = self.profile_items();
        items.extend([
            MenuItem::Separator,
            StandardItem {
                label: "Open".into(),
                icon_name: "applications-internet".into(),
                activate: Box::new(|tray: &mut Self| tray.open_dashboard()),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Refresh".into(),
                icon_name: "view-refresh".into(),
                activate: Box::new(|tray: &mut Self| tray.send(Command::Refresh)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|tray: &mut Self| tray.send(Command::Quit)),
                ..Default::default()
            }
            .into(),
        ]);
        items
    }
}
