## NZXT CLI

A simple NZXT tool for managing fans and LEDs.

## Install

To build the tool from source code, You need:

* Rust: Version specified in [Cargo.toml](./Cargo.toml?#L5) or greater.
* `libudev` and `libcap`

```bash
cargo install --path . --locked
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

## License

Licensed under MIT license ([LICENSE](./LICENSE) or <https://opensource.org/licenses/MIT>)
