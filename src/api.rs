use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ureq::Agent;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceProfiles {
    pub serial: String,
    pub product: String,
    pub profiles: Vec<String>,
    pub active: Option<usize>,
}

#[derive(Deserialize)]
struct DevicesResponse {
    #[serde(default)]
    devices: HashMap<String, DeviceEntry>,
}

#[derive(Deserialize)]
struct DeviceEntry {
    #[serde(rename = "Product", default)]
    product: String,
    #[serde(rename = "Serial", default)]
    serial: String,
    #[serde(rename = "Hidden", default)]
    hidden: bool,
    #[serde(rename = "GetDevice", default)]
    device: Option<DeviceDetails>,
}

#[derive(Deserialize)]
struct DeviceDetails {
    #[serde(rename = "userProfiles", default)]
    user_profiles: Option<BTreeMap<String, UserProfile>>,
}

#[derive(Deserialize)]
struct UserProfile {
    #[serde(rename = "Active", default)]
    active: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangeProfileRequest<'a> {
    device_id: &'a str,
    user_profile_name: &'a str,
}

#[derive(Deserialize)]
struct StatusResponse {
    #[serde(default)]
    status: i32,
    #[serde(default)]
    message: String,
}

pub struct OpenLinkHubClient {
    agent: Agent,
    base_url: String,
}

impl OpenLinkHubClient {
    pub fn new(base_url: String) -> Self {
        let agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .into();
        Self {
            agent,
            base_url,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn device_profiles(&self) -> Result<Vec<DeviceProfiles>, String> {
        let response: DevicesResponse = self
            .agent
            .get(format!("{}/api/devices/", self.base_url))
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;

        let mut devices: Vec<DeviceProfiles> = response
            .devices
            .into_iter()
            .filter(|(_, d)| !d.hidden)
            .filter_map(|(key, d)| {
                let profiles = d.device?.user_profiles?;
                if profiles.is_empty() {
                    return None;
                }
                let active = profiles.values().position(|p| p.active);
                Some(DeviceProfiles {
                    serial: if d.serial.is_empty() { key } else { d.serial },
                    product: d.product,
                    profiles: profiles.into_keys().collect(),
                    active,
                })
            })
            .collect();

        devices.sort_by(|a, b| a.product.cmp(&b.product).then(a.serial.cmp(&b.serial)));
        Ok(devices)
    }

    pub fn change_profile(&self, serial: &str, profile: &str) -> Result<(), String> {
        let response: StatusResponse = self
            .agent
            .post(format!("{}/api/userProfile/change", self.base_url))
            .send_json(ChangeProfileRequest {
                device_id: serial,
                user_profile_name: profile,
            })
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;

        match response.status {
            1 => Ok(()),
            _ => Err(response.message),
        }
    }
}
