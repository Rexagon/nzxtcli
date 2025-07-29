use std::sync::OnceLock;

use hidapi::HidApi;

pub use self::controller::{ChannelDeviceInfo, LedMode, NZXTHue2Controller, RgbChannel};
pub use self::types::{Color, Version};

mod controller;
mod types;

pub const NZXT_VID: u16 = 0x1E71;

pub static BIN_VERSION: &str = env!("NZXTCLI_API_VERSION");

pub fn version_string() -> &'static str {
    static STRING: OnceLock<String> = OnceLock::new();
    STRING.get_or_init(|| format!("(release {BIN_VERSION})"))
}

pub fn find_controllers<'a>(api: &'a HidApi) -> Vec<NZXTHue2Controller<'a>> {
    let known = NZXTHue2Controller::known_controllers();
    let mut result = Vec::new();

    for device in api.device_list() {
        if device.vendor_id() != NZXT_VID {
            continue;
        }

        let span = tracing::debug_span!(
            "device",
            id = %format_args!("{:04x}:{:04x}", device.vendor_id(), device.product_id()),
            name = device.product_string().unwrap_or("<unknown>"),
        );
        let _guard = span.enter();

        if let Some(&(name, rgb_channels, fan_channels)) = known.get(&device.product_id()) {
            match NZXTHue2Controller::new(api, device, name, rgb_channels, fan_channels) {
                Ok(entry) => {
                    tracing::debug!("found NZXT device");
                    result.push(entry)
                }
                Err(e) => {
                    tracing::debug!("failed to create controller: {e:?}");
                    continue;
                }
            }
        } else {
            tracing::debug!("skipping unknown device")
        }
    }

    result
}
