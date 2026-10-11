mod api;
mod tray;

use std::process::ExitCode;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use clap::Parser;
use ksni::blocking::{Handle, TrayMethods};

use crate::api::OpenLinkHubClient;
use crate::tray::{Command, OpenLinkHubTray};

/// System tray icon for OpenLinkHub
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Host or IP address of the OpenLinkHub service
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    /// Port of the OpenLinkHub service
    #[arg(long, default_value_t = 27003)]
    port: u16,

    /// Refresh interval in seconds
    #[arg(long, default_value_t = 30)]
    interval: u64,
}

fn refresh(client: &OpenLinkHubClient, handle: &Handle<OpenLinkHubTray>) {
    let result = client.device_profiles();
    handle.update(|tray| match result {
        Ok(devices) => {
            tray.devices = devices;
            tray.error = None;
        }
        Err(e) => tray.error = Some(e),
    });
}

fn main() -> ExitCode {
    let args = Args::parse();
    let client = OpenLinkHubClient::new(format!("http://{}:{}", args.ip, args.port));
    let (tx, rx) = mpsc::channel();

    let tray = OpenLinkHubTray {
        dashboard_url: client.base_url().to_string(),
        devices: Vec::new(),
        error: None,
        commands: tx,
    };

    let handle = match tray.assume_sni_available(true).spawn() {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to start tray: {e}");
            return ExitCode::FAILURE;
        }
    };

    refresh(&client, &handle);

    let interval = Duration::from_secs(args.interval.max(1));
    loop {
        match rx.recv_timeout(interval) {
            Ok(Command::Refresh) | Err(RecvTimeoutError::Timeout) => {}
            Ok(Command::ChangeProfile { serial, profile }) => {
                if let Err(e) = client.change_profile(&serial, &profile) {
                    eprintln!("Failed to change profile to {profile} on {serial}: {e}");
                }
            }
            Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => break,
        }
        if handle.is_closed() {
            break;
        }
        refresh(&client, &handle);
    }

    handle.shutdown().wait();
    ExitCode::SUCCESS
}
