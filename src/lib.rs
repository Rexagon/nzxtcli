use std::sync::{Mutex, OnceLock};

use hidapi::HidApi;

pub use self::controller::{ChannelDeviceInfo, LedMode, NZXTHue2Controller, RgbChannel};
pub use self::types::Color;

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
    let result = Mutex::new(Vec::new());

    std::thread::scope(|scope| {
        for device in api.device_list() {
            if device.vendor_id() != NZXT_VID {
                continue;
            }

            scope.spawn(|| {
                if let Some(&(name, rgb_channels, fan_channels)) = known.get(&device.product_id()) {
                    match NZXTHue2Controller::new(api, device, name, rgb_channels, fan_channels) {
                        Ok(entry) => result.lock().unwrap().push(entry),
                        Err(e) => {
                            eprintln!("failed to create controller: {e:?}");
                        }
                    }
                }
            });
        }
    });

    result.into_inner().unwrap()
}
