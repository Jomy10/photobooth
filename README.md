<div align="center">
  <h1>📸 Photobooth</h1>
  <p>A photobooth application for the raspberry pi and pi camera</p>
</div><br/>

https://github.com/user-attachments/assets/19b1c76b-2000-4e29-a9ea-37e35ef21a2d

**Navigation**
- [Setup](#setup)
  - [Hardware](#hardware)
  - [Software](#software)
- [Install](#install)
  - [Download](#download)
  - [Building from source](#building-from-source)
- [Configuration](#configuration)
  - [Automatically mounting USB devices](#automatically-mounting-usb-devices)
  - [Permissions](#permissions)
  - [Boot config](#boot-config)
- [Additional notes](#additional-notes)
- [Questions](#questions)
- [License](#license)

# Setup

## Hardware

The application has been tested and works on the following configuration:
- [Raspberry Pi 4B (2GB RAM model)](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/)
- [Wimaxit M728 7" Touch Screen](https://wimaxit.com/products/wimaxit-raspberry-pi-7-touch-screen-display-monitor-1024x600-usb-powered-hdmi-screen-monitor-ips-178-with-rear-speakers-stand-for-raspberry-4-3-2-laptop-pc)
- [Raspberry Pi Camera Module v3](https://www.raspberrypi.com/products/camera-module-3/)

All three products come with the necesarry pieces to connect them together.

This application has not been tested on other hardware, but I am open to pull
requests adding support for them, or simply letting me know this application
also works on a different hardware configuration.

In addition to the pi, screen and camera, a USB stick also needs to be connected and mounted on the pi (see the [configuration section for auto mounting USB storage devices](#automatically-mounting-usb-devices)). Images will be saved on this USB stick for easy transfering to another computer.

![back](https://github.com/user-attachments/assets/f9275a29-b301-4121-b4ad-38c7947d53bb)
![front](https://github.com/user-attachments/assets/48ee3539-b253-4d08-8f4f-0b7b884b3347)

## Software

1. Install a new copy of **Raspbery Pi OS Lite (64-bit)** on your Raspberry Pi's SD-card
2. [For convenience, you can change the font size of the terminal using `sudo dpkg-reconfigure console-setup`](https://www.raspberrypi-spy.co.uk/2014/04/how-to-change-the-command-line-font-size/)
3. Enable auto-login on boot:
    - `sudo raspi-config`
    - go to *System Options*
    - go to *Boot / Auto Login*
    - Select *Console Autologin*
4. [Edit the boot config](#boot-config)
5. [Install the photobooth software](#Install); either [build it from soure](#building-from-source) or [download the latest binary](#download)
6. [Configure the photobooth software](#configuration)
7. Start photobooth at boot:
    add the following to the end of ~/.bashrc:
    ```sh
    # Replace this with the path to your config file (or comment it out to
    # not apply any configuration)
    export PH_CONFIG=config.yaml

    # Replace this with the path to the photobooth executable
    # This line will start the application
    ./photobooth
    ```
8. Reboot the pi: `reboot`

# Install

Install the required dependencies:

```sh
sudo apt-get install \
  libinput-dev \
  libcamera-dev \
  libdrm-dev
```

Now either [download](#download) the latest binary, or [build from source](#building-from-source).

## Download

Pre-compiled binaries can be found in the [the latest release](https://github.com/Jomy10/photobooth/releases/latest).

You can also use this one-liner to download the latest release:
```sh
wget \
    "$( \
        curl -s https://api.github.com/repos/jomy10/photobooth/releases/latest |
        jq -r '.assets[] | select(.name=="Photobooth") | .browser_download_url' \
    )" -o photobooth && chmod +x photobooth
```

The command requires `jq` to parse json, which can be downloaded with `sudo apt-get install jq`.

## Building from source

This program uses the experimental `mpmc` channels, so it requires nightly.

Download Noto Emoji and Space Mono Bold [Google fonts](https://fonts.google.com/noto/specimen/Noto+Emoji?selection.family=Noto+Color+Emoji|Noto+Emoji:wght@300..700|Space+Mono:ital,wght@0,400;0,700;1,400;1,700)
(or change the used fonts in code) and place them in the root directory of the project.

*Download Noto Emoji, not Noto Color Emoji. Color emojis are not supported*.

```sh
cargo +nightly build --release
```

The application will now be located at `target/release/photobooth`.

# Configuration

The `PH_CONFIG` environment variable can be set to point to a config file (yaml).
Definition and defaults can be found in [config.rs](./src/config.rs). Here,
translations can be added tailored to the end user of the photobooth.

## Automatically mounting USB devices

The photobooth application expects a USB storage device to be mounted. Using the
[automount](./automount) utility in this repository, you can set up a service
to automatically mount all USB devices connected to the PI.

## Permissions

Make sure the user running the application has the write rights to /var/log and the mount point of your USB device.

## Boot config

This should be present in /boot/firmware/config.txt

```
# Automatically load overlays for detected cameras
camera_auto_detect=1

# Automatically load overlays for detected DSI displays
display_auto_detect=1

# Automatically load initramfs files, if found
auto_initramfs=1

# Enable DRM VC4 V3D driver
dtoverlay=vc4-kms-v3d
max_framebuffers=2

dtoverlay=cma,cma-256
```

Reboot if necessary:

```sh
sudo reboot
```

# Additional notes

- I have used a PI with 2GB of RAM, 1GB might not be enough.

# Questions

For any questions, feel free to [open an issue](https://github.com/Jomy10/photobooth/issues/new/choose).

# License

[GNU GPL](LICENSE).

Photobooth: Photobooth software for touch screen devices<br/>
Copyright (C) 2024 Jonas Everaert

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

[Dependencies may be licensed differently](LICENSE_DEPENDENCIES).
