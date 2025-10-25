# Automounting USB devices

1. Change the `user` variable in `automount.sh` to the user running the photobooth application
2. Copy `automount.sh` to /usr/local/bin
3. Copy `automount@.service` to /etc/systemd/system
4. Copy `99-automount.rules` to /etc/udev/rules.d
5. Reload configurations
```sh
sudo systemctl daemon-reload
sudo udevadm control --reload-rules
```
