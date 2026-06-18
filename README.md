# EchoInput

EchoInput is a tool to capture and display keystrokes.

## Linux Compilation Dependencies

To compile this project on Linux, you need development packages for both X11 and Wayland input processing libraries, along with `pkg-config`.

### Ubuntu/Debian
```bash
sudo apt install libx11-dev libxtst-dev libxext-dev libinput-dev libxkbcommon-dev pkg-config
```

### Fedora/RHEL
```bash
sudo dnf install libX11-devel libXtst-devel libXext-devel libinput-devel libxkbcommon-devel pkgconfig
```

### Arch Linux
```bash
sudo pacman -S libx11 libxtst libxext libinput libxkbcommon pkgconf
```

## Running on Wayland

Because the Wayland input hook reads raw hardware keyboard events directly from `/dev/input/event*`, it requires elevated permissions to access these input devices.

There are two ways to run it properly on Wayland:

### 1. Run with Sudo (Quickest)
Since `sudo` scrubs the user's environment by default, you must explicitly pass `WAYLAND_DISPLAY` through so the backend selector knows it is running under a Wayland session:
```bash
sudo WAYLAND_DISPLAY=$WAYLAND_DISPLAY ./target/debug/echoinput
```

### 2. Run without Sudo (Recommended for Dev)
You can grant your normal user permissions to read input devices by adding yourself to the `input` group:
```bash
# Add yourself to the group (only needs to be run once)
sudo usermod -aG input $USER
```
*Note: You must log out of your desktop session and log back in for the group membership to take effect.* 

Once logged back in, you can run the binary directly:
```bash
./target/debug/echoinput
```
