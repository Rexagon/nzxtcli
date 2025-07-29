use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Result;
use hidapi::HidResult;

use crate::types::Color;

pub struct NZXTHue2Controller<'a> {
    device: hidapi::HidDevice,
    info: &'a hidapi::DeviceInfo,
    name: &'static str,
    rgb_channels: Vec<RgbChannel>,
}

/// Name, RGB Channels, Fan Channels
type ControllerBriefInfo = (&'static str, usize, usize);

impl<'a> NZXTHue2Controller<'a> {
    pub fn known_controllers() -> &'static HashMap<u16, ControllerBriefInfo> {
        static INFO: OnceLock<HashMap<u16, ControllerBriefInfo>> = OnceLock::new();
        INFO.get_or_init(|| {
            HashMap::from_iter([
                // Fan controller
                (0x2009, ("NZXT RGB & Fan Controller", 2, 3)),
                (0x2010, ("NZXT RGB & Fan Controller", 2, 3)),
                (0x200E, ("NZXT RGB & Fan Controller", 2, 3)),
                // Fan controller (6-channel)
                (0x2011, ("NZXT RGB & Fan Controller", 6, 3)),
                (0x2019, ("NZXT RGB & Fan Controller", 6, 3)),
                (0x2020, ("NZXT RGB & Fan Controller", 6, 3)),
                (0x201F, ("NZXT RGB & Fan Controller", 6, 3)),
                (0x2022, ("NZXT RGB & Fan Controller 2024", 6, 3)),
                (0x201B, ("NZXT B650E Motherboard", 6, 3)),
                // HUE 2
                (0x2001, ("NZXT Hue 2", 4, 0)),
                (0x2002, ("NZXT Hue 2 Ambient", 2, 0)),
                (0x2005, ("NZXT Hue 2 Motherboard", 2, 3)),
                (0x200B, ("NZXT Hue 2 Motherboard", 2, 3)),
                // Kraken
                (0x2007, ("NZXT Kraken X3 Series", 3, 0)),
                (0x2014, ("NZXT Kraken X3 Series RGB", 3, 0)),
                (0x3012, ("NZXT Kraken 2024 ELITE Series RGB", 2, 2)),
                // RGB Controller
                (0x2012, ("NZXT RGB Controller", 3, 0)),
                (0x2021, ("NZXT RGB Controller", 3, 0)),
                // Smart Device
                (0x2006, ("NZXT Smart Device V2", 2, 3)),
                (0x200D, ("NZXT Smart Device V2", 2, 3)),
                (0x200F, ("NZXT Smart Device V2", 2, 3)),
            ])
        })
    }

    pub fn new(
        api: &'a hidapi::HidApi,
        info: &'a hidapi::DeviceInfo,
        name: &'static str,
        rgb_channels: usize,
        _fan_channels: usize,
    ) -> Result<Self> {
        let device = api.open_path(info.path())?;
        let rgb_channels = get_channels_info(&device, rgb_channels)?;

        Ok(Self {
            device,
            info,
            name,
            rgb_channels,
        })
    }

    pub fn info(&self) -> &hidapi::DeviceInfo {
        self.info
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn rgb_channels(&self) -> &[RgbChannel] {
        &self.rgb_channels
    }

    pub fn set_fixed_color(&self, color: Color) -> Result<()> {
        let mut colors = Vec::new();
        for (i, channel) in self.rgb_channels.iter().enumerate() {
            colors.resize(channel.led_count, color);
            set_channel_leds(&self.device, i, &colors)?;
        }
        Ok(())
    }
}

fn get_channels_info(
    device: &hidapi::HidDevice,
    rgb_channels: usize,
) -> HidResult<Vec<RgbChannel>> {
    let mut buffer = [0u8; 64];
    buffer[0] = 0x20;
    buffer[1] = 0x03;
    device.write(&buffer)?;

    // TODO: Add some iterations check
    loop {
        let ret_val = device.read(&mut buffer)?;
        if ret_val == 64 && buffer[0] == 0x21 && buffer[1] == 0x03 {
            break;
        }
    }

    let mut result = Vec::new();
    for channel in 0..rgb_channels {
        let start = 0x0f + (HUE_2_NUM_CHANNELS * channel);
        let mut channel_info = RgbChannel::default();

        for dev in 0..HUE_2_NUM_CHANNELS {
            let id = buffer[start + dev];
            let (led_count, name) = match id {
                0x01 => (10, "Hue 1 strip"),
                0x02 => (8, "Aer 1 fan"),
                0x04 => (10, "Hue 2 strip (10 LEDs)"),
                0x05 => (8, "// Hue 2 strip (8 LEDs)"),
                0x06 => (6, "Hue 2 strip (6 LEDs)"),
                0x08 => (14, "Hue 2 Cable Comb (14 LEDs)"),
                0x09 => (15, "Hue 2 Underglow (300mm) (15 LEDs)"),
                0x0a => (10, "Hue 2 Underglow (200mm) (10 LEDs)"),
                0x0b => (8, "Aer 2 fan (120mm)"),
                0x0c => (8, "Aer 2 fan (140mm)"),
                0x10 => (8, "Kraken X3 ring"),
                0x11 => (1, "Kraken X3 logo"),
                0x13 => (18, "F120 RGB fan (120mm)"),
                0x14 => (18, "F140 RGB fan (140mm)"),
                0x15 => (20, "F120 RGB Duo fan (120mm)"),
                0x16 => (20, "F140 RGB Duo fan (140mm)"),
                0x17 => (8, "F120 RGB Core fan (120mm)"),
                0x18 => (8, "F140 RGB Core fan (140mm)"),
                0x19 => (8, "F120 RGB Core fan case version (120mm)"),
                0x1d => (24, "F360 RGB Core Fan Case Version (360mm)"),
                0x1e => (24, "Kraken Elite Ring"),
                _ => (0, "<unknown>"),
            };

            if led_count == 0 {
                continue;
            }

            channel_info.led_count += led_count as usize;
            channel_info.devices[dev] = ChannelDeviceInfo {
                id,
                name,
                led_count,
            };
        }

        result.push(channel_info);
    }

    Ok(result)
}

fn set_channel_leds(
    device: &hidapi::HidDevice,
    channel: usize,
    mut colors: &[Color],
) -> HidResult<()> {
    let mut group = 0;
    while !colors.is_empty() {
        let count = std::cmp::min(colors.len(), 20);
        send_direct(device, channel, group, &colors[..count])?;
        colors = &colors[count..];
        group += 1;
    }
    send_apply(device, channel)
}

fn send_direct(
    device: &hidapi::HidDevice,
    channel: usize,
    group: u8,
    color_data: &[Color],
) -> HidResult<()> {
    let mut buffer = [0u8; 64];
    buffer[0x00] = 0x22;
    buffer[0x01] = 0x10 | group;
    buffer[0x02] = 0x01u8 << channel;
    buffer[0x03] = 0x00;
    buffer[0x04..0x04 + (color_data.len() * 3)].copy_from_slice(Color::wrap_slice(color_data));
    device.write(&buffer)?;
    Ok(())
}

fn send_apply(device: &hidapi::HidDevice, channel: usize) -> HidResult<()> {
    let mut buffer = [0u8; 64];
    buffer[0x00] = 0x22;
    buffer[0x01] = 0xa0;
    buffer[0x02] = 0x01u8 << channel;
    buffer[0x04] = 0x01;
    buffer[0x07] = 0x28;
    buffer[0x0a] = 0x80;
    buffer[0x0c] = 0x32;
    buffer[0x0f] = 0x01;
    device.write(&buffer)?;
    Ok(())
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RgbChannel {
    pub led_count: usize,
    pub devices: [ChannelDeviceInfo; HUE_2_NUM_CHANNELS],
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ChannelDeviceInfo {
    pub id: u8,
    pub name: &'static str,
    pub led_count: u8,
}

#[repr(u8)]
pub enum LedMode {
    Fixed = 0x00,
    Fading = 0x01,
    Spectrum = 0x02,
    Marquee = 0x03,
    CoverMarquee = 0x04,
    Alternating = 0x05,
    Pulsing = 0x06,
    Breathing = 0x07,
    Candle = 0x08,
    StarryNight = 0x09,

    RainbowFlow = 0x0b,
    SuperRainbow = 0x0c,
    RainbowPulse = 0x0d,
}

const HUE_2_NUM_CHANNELS: usize = 6;
