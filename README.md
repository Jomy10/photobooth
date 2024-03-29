## Setup

### Hardware

### Software

1. Install a new copy of **Raspbery Pi OS Lite (64-bit)** on your Raspberry Pi's SD-card
2. [For convenience, you can change the font size of the terminal using `sudo dpkg-reconfigure console-setup`](https://www.raspberrypi-spy.co.uk/2014/04/how-to-change-the-command-line-font-size/)
3. Enable auto-login on boot:
    - `sudo raspi-config`
    - go to *System Options*
    - go to *Boot / Auto Login*
    - Select *Console Auutologin*
4. [Install the photobooth software](#Install); either [build it from soure](#building-from-source) or [download the latest binary](#download)
5. Start photobooth at boot: <!-- TODO -->
6. Reboot the pi: `reboot`

## Install

### Download

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
```

