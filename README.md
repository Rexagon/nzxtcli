## NZXT CLI

A simple NZXT tool for managing fans and LEDs.

## Install

To build the tool from source code, You need:

* Rust: Version specified in [Cargo.toml](./Cargo.toml?#L5) or greater.
* `libudev` and `libcap`

```bash
cargo install --path . --locked
```

After that you need to install udev rules:
```bash
# Copy rules file
sudo cp ./contrib/60-nzxtcli.rules /usr/lib/udev/rules.d/60-nzxtcli.rules
# Reload rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## How to use

List all devices:
```bash
nzxtcli list
```

<details><summary><b>Output</b></summary>
<p>

```json
[
  {
    "vendor_id": 7793,
    "vendor_id_hex": "1e71",
    "product_id": 8210,
    "product_id_hex": "2012",
    "name": "NZXT RGB Controller",
    "firmware_version": "1.5.0",
    "rgb_channels": [
      {
        "id": 0,
        "led_count": 18,
        "devices": [
          {
            "id": 0,
            "id_hex": "00",
            "name": "F140 RGB fan (140mm)",
            "led_count": 18
          }
        ]
      },
      {
        "id": 1,
        "led_count": 18,
        "devices": [
          {
            "id": 0,
            "id_hex": "00",
            "name": "F140 RGB fan (140mm)",
            "led_count": 18
          }
        ]
      },
      {
        "id": 2,
        "led_count": 0,
        "devices": []
      }
    ]
  },
  {
    "vendor_id": 7793,
    "vendor_id_hex": "1e71",
    "product_id": 8225,
    "product_id_hex": "2021",
    "name": "NZXT RGB Controller",
    "firmware_version": "1.5.0",
    "rgb_channels": [
      {
        "id": 0,
        "led_count": 8,
        "devices": [
          {
            "id": 0,
            "id_hex": "00",
            "name": "F120 RGB Core fan (120mm)",
            "led_count": 8
          }
        ]
      },
      {
        "id": 1,
        "led_count": 8,
        "devices": [
          {
            "id": 0,
            "id_hex": "00",
            "name": "F120 RGB Core fan (120mm)",
            "led_count": 8
          }
        ]
      },
      {
        "id": 2,
        "led_count": 0,
        "devices": []
      }
    ]
  }
]
```
</p>
</details>

Set the same color for all LEDs on all devices:
```bash
nzxtcli set-color ffaabb
```

Sync LEDs color to the CPU temp (or any other temperatur sensor).
Use `sensors` to find preferred temperature source, then run
```bash
for i in /sys/class/hwmon/hwmon*/temp*_input; do
  echo "$(<$(dirname $i)/name): $(cat ${i%_*}_label 2>/dev/null || echo $(basename ${i%_*})) $(readlink -f $i)";
done
```
to find path to desired file.

Then run this tool:
```bash
nzxtcli cpu-temp \
    /sys/devices/pci0000:00/0000:00:18.3/hwmon/hwmon4/temp1_input \
    --interval 100ms \
    --base 20 \
    --warn 90
```

> You can create a systemd service for this command, see [the example](./contrib/cpu-temp.service).

## License

Licensed under MIT license ([LICENSE](./LICENSE) or <https://opensource.org/licenses/MIT>)
