use std::io::IsTerminal;

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

fn print_json<T: Serialize>(output: T) -> Result<()> {
    let output = if std::io::stdin().is_terminal() {
        serde_json::to_string_pretty(&output)
    } else {
        serde_json::to_string(&output)
    }?;

    println!("{output}");
    Ok(())
}
