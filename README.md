# i2c-tools Business Card

> This project is inspired by [I2CDetect Business Card](https://hackaday.io/project/196148-i2cdetect-business-card).

In this project, we are using a RISC-V based CH32V203 MCU to scan the I2C bus. With a USB device controller, we can develop PC-side software to enable additional applications, such as `i2ctransfer`, and more.

### TODO

- [x] Basic one-time scan
- [x] Continuous scan
- [ ] PC-side software support
- [ ] linux usb to i2c adapter driver
- [ ] linux MFD (Multi-Function Device) driver (GPIO/I2C/SPI/UART)

### Build

install dependencies

```bash
rustup target add riscv32imac-unknown-none-elf

cargo install --locked wlink
```

```bash
cargo build -r
```

### Flash firmware

With a WCH-Link(E) probe connected to your target and then:

```bash
cargo run -r
```

### Run examples

see examples dir

```bash
cargo run -r --example i2c-ssd1306
```
