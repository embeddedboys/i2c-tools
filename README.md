# i2c-tools

> This project is inspired by [I2CDetect Business Card](https://hackaday.io/project/196148-i2cdetect-business-card).

In this project, we are using a RISC-V-based CH32V203 MCU to scan the I2C bus. With a USB device controller, we can develop PC-side software to enable additional applications, such as `i2ctransfer`, and more.

### TODO

- [x] Basic one-time scan
- [ ] Continuous scan
- [ ] PC-side software support

### Build

install dependencies

```bash
rustup target add riscv32imac-unknown-none-elf

cargo install --locked wlink
```

```bash
cargo build --release
```

### Flash firmware

With a WCH-Link/E probe connected to your target and then:

```bash
cargo run -r
```

### Run examples

```bash
cargo run -r --example i2c-ssd1306
```
