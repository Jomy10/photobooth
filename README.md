## Software

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
    ./photobooth
    ```
6. Reboot the pi: `reboot`


dependencies:
```sh
sudo apt-get install \
  libinput-dev \
  libcamera-dev \
  libdrm-dev
```
<!--- software-properties-common ?
- libfonconfig-dev ?-->

## Building

This program uses the experimental `mpmc` channels, so it requires nightly.

```sh
cargo +nightly build --release
```

## Automatically mounting USB devices

See [autmount](./automount).

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
