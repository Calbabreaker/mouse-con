# Linux Mouse + Keyboard Virtual Controller

Note: Wayland is not supported.

Make sure the uinput kernel module is loaded:

```
sudo modprobe uinput
```

The script must have access to /dev/uinput so do this to make and add yourself to the uinput group:

```
sudo groupadd uinput
sudo usermod -aG uinput "$USER"
sudo chmod g+rw /dev/uinput
sudo chgrp uinput /dev/uinput
```

This script will try to use xbanish to hide the cursor.
If you don't have xbanish, make sure to hide the cursor in the app.
