# Smart Home Gateway firmware prototype

This directory provides code for the firmware of a smart home gateway prototype.

ADD PICTURE

## Prerequisites

In order to build the firmware you need to install the Rust programming language (see [Rust install](https://www.rust-lang.org/tools/install)) and Docker. Mind that Rust implicitly expects a linker to be installed on your system.

After installing Rust, run:

- `cargo install cross --git https://github.com/cross-rs/cross` for easy cross compilation;
- `rustup default 1.75.0` to get the Rust toolchain version required to build this project binaries;
- `rustup target add wasm32-wasi` to install the wasm build target needed by virtual device drivers.

You can skip the `cross` and Docker setup steps if you are installing Rust on a Raspberry Pi for the LoRa proof of concept illustrated below.

## Running the smart gateway mockup

Test with following commands ***from this directory***

```bash
cargo build -p virt_dev --target wasm32-wasi --release
```

```bash
cargo run -p smart_gw
```

This will run a mockup version of the gateway where reception of messages from LoRa is emulated.

## LoRa proof of concept

If you have two [Raspberry Pi 3B](https://en.wikipedia.org/wiki/Raspberry_Pi) with [Dragino LoRa GPS HAT](https://www.dragino.com/downloads/downloads/LoRa-GPS-HAT/LoRa_GPS_HAT_UserManual_v1.0.pdf) modules, we also provide code for a physical proof of concept. Install Raspberry Pi OS (tested on [this version]((https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2023-10-10/))) and make sure the SPI interface is enabled with `sudo raspi-config`.

You can either clone this repo and install rust on the Raspberry Pis (as above, minus the installation of `cross` and Docker) to automatically build & run for their architecture, or you can [cross compile](https://github.com/cross-rs/cross) the binaries with

```bash
cross build -p <crate> --target aarch64-unknown-linux-gnu --release
```

and transfer them over via ssh with `scp` or using an USB drive. When cross-compiling, the output binaries can be found under `target/aarch64-unknown-linux-gnu/release/`. For this proof of concept we provide the following executables:

- The `phy_dev` crate provides a binary to send LoRa transmissions emulating multiple end-devices. Transfer the binary on the first Raspberry Pi and run it with `./phy_dev`.

- The `smart_gw` crate provides a binary to run the smart home gateway. Before running the smart gateway, build the virtual device driver as in the previous section (`cargo build -p virt_dev --target wasm32-wasi --release`). If you are cross-compiling, transfer the virtual device driver wasm binary under the directory structure `target/wasm32-wasi/release/virt_dev.wasm` where you placed the smart gateway binary. Now you can run the smart gateway in LoRa mode with `.smart_gw --lora`.
