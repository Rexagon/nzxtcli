use std::io::{IsTerminal, Read, Seek};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hidapi::HidApi;
use nzxtcli::{Color, find_controllers};
use serde::Serialize;

fn main() -> Result<()> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        // Enable backtraces on panics by default.
        // SAFETY: There is only a single thread at the moment.
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        // Disable backtraces in libraries by default
        // SAFETY: There is only a single thread at the moment.
        unsafe { std::env::set_var("RUST_LIB_BACKTRACE", "0") };
    }

    match App::parse().cmd {
        SubCmd::List(cmd) => cmd.run(),
        SubCmd::SetColor(cmd) => cmd.run(),
        SubCmd::CpuTemp(cmd) => cmd.run(),
    }
}

/// A simple NZXT tool for managing fans and LEDs.
#[derive(Parser)]
#[clap(version = nzxtcli::version_string())]
#[clap(subcommand_required = true)]
struct App {
    #[clap(subcommand)]
    cmd: SubCmd,
}

#[derive(Subcommand)]
enum SubCmd {
    List(CmdList),
    SetColor(CmdSetColor),
    CpuTemp(CmdCpuTemp),
}

/// List all supported NZXT devices.
#[derive(Parser)]
struct CmdList {}

impl CmdList {
    fn run(self) -> Result<()> {
        let api = HidApi::new().context("failed to initialize HID api")?;
        let controllers = find_controllers(&api);

        let mut info = Vec::with_capacity(controllers.len());
        for controller in controllers {
            let vendor_id = controller.info().vendor_id();
            let product_id = controller.info().product_id();

            let rgb_channels = controller.rgb_channels();
            let rgb_channels = rgb_channels
                .iter()
                .enumerate()
                .map(|(id, channel)| {
                    let devices = channel
                        .devices
                        .iter()
                        .enumerate()
                        .filter_map(|(id, device)| {
                            if device.led_count == 0 {
                                None
                            } else {
                                Some(serde_json::json!({
                                    "id": id,
                                    "id_hex": format!("{id:02x}"),
                                    "name": device.name,
                                    "led_count": device.led_count,
                                }))
                            }
                        })
                        .collect::<Vec<_>>();

                    serde_json::json!({
                        "id": id,
                        "led_count": channel.led_count,
                        "devices": devices,
                    })
                })
                .collect::<Vec<_>>();

            info.push(serde_json::json!({
                "vendor_id": vendor_id,
                "vendor_id_hex": format!("{vendor_id:04x}"),
                "product_id": product_id,
                "product_id_hex": format!("{product_id:04x}"),
                "name": controller.name(),
                "rgb_channels": rgb_channels,
            }));
        }

        print_json(info).unwrap();
        Ok(())
    }
}

/// Set the same color for all devices and channels.
#[derive(Parser)]
struct CmdSetColor {
    #[clap()]
    color: Color,
}

impl CmdSetColor {
    fn run(self) -> Result<()> {
        let api = HidApi::new().context("failed to initialize HID api")?;
        let controllers = find_controllers(&api);

        for controller in controllers {
            controller
                .set_fixed_color(self.color)
                .with_context(|| format!("failed to set color for {}", controller.name()))?;
        }

        Ok(())
    }
}

/// Sync LED colors with the CPU temp.
#[derive(Parser)]
struct CmdCpuTemp {
    /// Full path of temperature sysfs path.
    #[clap()]
    hwmon_path: PathBuf,

    #[clap(long, value_parser = humantime::parse_duration)]
    interval: Duration,

    /// Base temperature for where to start the ramp (in degrees celsius).
    #[clap(long, default_value_t = 0)]
    base: u64,

    /// Threshold temperature to display the hottest color (in degrees celsius).
    #[clap(long, default_value_t = 80)]
    warn: u64,
}

impl CmdCpuTemp {
    fn run(mut self) -> Result<()> {
        const MIN_TEMP: Duration = Duration::from_millis(100);

        anyhow::ensure!(
            self.base < self.warn,
            "'warn' temperature must be greater than the 'base'"
        );

        let ramp = [
            (0u64, Color::new(0x07, 0x05, 0x02)),
            (250, Color::new(0x1B, 0x2E, 0x04)),
            (600, Color::new(0x39, 0x20, 0x02)),
            (700, Color::new(0x79, 0x09, 0x00)),
            (900, Color::new(0xff, 0x00, 0x00)),
        ];

        self.interval = std::cmp::max(self.interval, MIN_TEMP);

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(self.hwmon_path)
            .context("failed to open `hwmon` file")?;

        let api = HidApi::new().context("failed to initialize HID api")?;
        let controllers = find_controllers(&api);

        let mut wait_until = Instant::now();
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            file.seek(std::io::SeekFrom::Start(0))?;
            file.read_to_end(&mut buffer)?;

            let temp = str::from_utf8(&buffer)?
                .trim()
                .parse::<u64>()?
                .clamp(self.base * SCALE, self.warn * SCALE);

            let normalized_temp = (temp - self.base * SCALE) / (self.warn - self.base);

            let mut color = ramp[0];
            let mut next_color = None;
            for ramp_item @ (threshold, ramp_color) in ramp {
                if normalized_temp >= threshold {
                    color = ramp_item;
                } else {
                    let t = (normalized_temp - color.0) * SCALE / (threshold - color.0);
                    next_color = Some((t, ramp_color));
                    break;
                }
            }

            let color = match next_color {
                None => color.1,
                Some((t, next_color)) => interpolate(color.1, next_color, t),
            };

            for controller in &controllers {
                controller.set_fixed_color(color)?;
            }

            wait_until += self.interval;
            std::thread::sleep(wait_until.duration_since(Instant::now()));
        }
    }
}

fn print_json<T: Serialize>(output: T) -> Result<()> {
    let output = if std::io::stdin().is_terminal() {
        serde_json::to_string_pretty(&output)
    } else {
        serde_json::to_string(&output)
    }?;

    println!("{output}");
    Ok(())
}

fn interpolate(mut a: Color, b: Color, mut t: u64) -> Color {
    t = u64::clamp(t, 0, SCALE);
    for (a, b) in std::iter::zip(a.inner_mut(), b.inner()) {
        *a = (((*a as u64) * (SCALE - t) + (*b as u64) * t) / SCALE) as u8;
    }
    a
}

const SCALE: u64 = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_interpolate() {
        let gray = interpolate(Color::BLACK, Color::WHITE, 500);
        println!("{gray:?}");
    }
}
