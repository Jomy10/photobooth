# Photobooth

A photobooth application for the Raspberry Pi and the Pi Camera Module.

**Navigation**
- [Setup](#setup)
    - [Hardware](#hardware)
    - [Software](#software)
- [Install](#install)
    - [Download](#download)
    - [Building from source](#building-from-source)
- [Configuration](#configuration)
- [Licensse](#license)

## Setup

### Hardware

The application has been tested and works on the following configuration:
- [Raspberry Pi 4B](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/)
- [Wimaxit M728 7" Touch Screen](https://wimaxit.com/products/wimaxit-raspberry-pi-7-touch-screen-display-monitor-1024x600-usb-powered-hdmi-screen-monitor-ips-178-with-rear-speakers-stand-for-raspberry-4-3-2-laptop-pc)
- [Raspberry Pi Camera Module v3](https://www.raspberrypi.com/products/camera-module-3/)

All three products come with the necesarry pieces to connect them together.

This application has not been tested on other hardware, but I am open to pull
requests adding support for them, or simply letting me know this application
also works on a different hardware configuration.

### Software

1. Install a new copy of **Raspbery Pi OS Lite (64-bit)** on your Raspberry Pi's SD-card
2. [For convenience, you can change the font size of the terminal using `sudo dpkg-reconfigure console-setup`](https://www.raspberrypi-spy.co.uk/2014/04/how-to-change-the-command-line-font-size/)
3. Enable auto-login on boot:
    - `sudo raspi-config`
    - go to *System Options*
    - go to *Boot / Auto Login*
    - Select *Console Auutologin*
4. [Install the photobooth software](#Install); either [build it from soure](#building-from-source) or [download the latest binary](#download)
5. Start photobooth at boot:
    add the following to the end of ~/.bashrc:
    ```sh
    # Replace this with the path to your config file (or comment it out to
    # not apply any configuration)
    export PH_CONFIG=config.yaml
    
    # Replace this with the path to the photobooth executable
    # This line will start the application
    Photobooth
    ```
6. Reboot the pi: `reboot`

## Install

### Download

Pre-compiled binaries can be found in the [the latest release](https://github.com/Jomy10/photobooth/releases/latest).

You can also use this one-liner to download the latest release:
```sh
wget \
    "$( \
        curl -s https://api.github.com/repos/jomy10/photobooth/releases/latest |
        jq -r '.assets[] | select(.name=="Photobooth") | .browser_download_url' \
    )" -o Photobooth &&
    chmod +x Photobooth
```

The command requires `jq` to parse json, which can be downloaded with `sudo apt-get install jq`

### Building from source

This chapter describes how to compile on the target Raspberry Pi.

```sh
# Install dependencies
sudo apt-get update && sudo apt-get upgrade
sudo apt-get install curl
curl -s https://archive.swiftlang.xyz/install.sh | sudo bash
sudo apt-get install swiftlang libcamera-apps libcairo2-dev libjpeg-dev

# Get the source code
git clone --recurse-submodules https://github.com/jomy10/photobooth
cd photobooth

# Compile the code
CONFIGURATION=release ./make.sh build

# The application is now located at .build/release/Photobooth
```

## Configuration

The `PH_CONFIG` environment variable can be set to point to a config file. An
example of this file:

```yaml
# Logs will be written out to this file
loggingPath: "photobooth_log.txt"
# Images will be saved in this folder
imagePath: "images"
# These will be displayed after an image has been taken
doneSentences:
  - All done!
  - Curious to see the result?
# Background color of the application
#          AARRGGBB
bgColor: 0xFF00FF00
```

### Removing old logs

You can add the following script to your home directory (in this example in `save_log.sh`):
```sh
if [ -f photobooth_log.txt ]; then
        if [ -f photobooth_log_prev2.txt ]; then
                rm photobooth_log_prev2.txt
        fi
        if [ -f photobooth_log_prev.txt ]; then
                mv photobooth_log_prev.txt photobooth_log_prev2.txt
        fi
        mv photobooth_log.txt photobooth_log_prev.txt
fi
```

Make it executable:
```sh
chmod +x save_log.sh
```

Then add the following line to cron using `crontab -e`:
```cron
@reboot /home/photobooth/save_log.sh
```

## License

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

Dependencies may be licensed differently:
- [cairo_jpeg](Sources/Graphisc/CairoJPEG): GNU LGPLv3
- [libdrm](deps/libdrm): MIT
- [swift-graphics](https://github.com/jomy10/swift-graphics): MIT
- [swift-cairo](https://github.com/jomy10/swift-cairo): MIT
- [swift-utils](https://github.com/fwcd/swift-utils): MIT
- [Yams](https://github.com/jpsim/Yams): MIT

