# i2c-tools

> This project is inspired by [I2CDetect Business Card](https://hackaday.io/project/196148-i2cdetect-business-card).

In this project, we are using a RISC-V-based CH32V203 MCU to scan the I2C bus. With a USB device controller, we can develop PC-side software to enable additional applications, such as `i2ctransfer`, and more.

### TODO

- [x] Basic one-time scan
- [ ] Continuous scan
- [ ] PC-side software support

### Build

```bash
cargo build --release
```

### Flash

[wlink](https://github.com/ch32-rs/wlink) needs to be installed:

```bash
cargo install --git https://github.com/ch32-rs/wlink
```

If you are using a WSL instance, you need to transport the usb device to the WSL instance via [usbipd]():
```shell
usbipd list

# 9-4    1a86:8010  WCH-Link, USB 串行设备 (COM28)                                Shared

usbipd bind -b 9-4
usbipd attach --wsl -b 9-4
```

With a WCH-Link probe connected to your target and then:

```bash
cargo run --release
```
