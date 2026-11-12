# Ubuntu installation

ClickPulse for Linux targets Ubuntu 22.04 and 24.04. Choose ARM64 for an Apple Silicon VM, or x64 for an Intel/AMD PC. Tests on one architecture do not establish support for the other.

## Install

Install the system libraries:

```sh
sudo apt update
sudo apt install libgtk-3-0 libwebkit2gtk-4.1-0 libayatana-appindicator3-1 libxdo3
```

Extract the complete portable ZIP to a permanent folder such as `~/.local/opt/ClickPulse`, then run `./clickpulse` from that folder. Keep its manifest and license files beside the program. The optional `.desktop` file needs an absolute Exec and Icon path before installing it into `~/.local/share/applications/`.

## Recording permission

Run this once as your normal desktop user, then sign out and back in:

```sh
sudo usermod -aG input "$USER"
```

Membership in `input` allows programs running as your account to read input devices. ClickPulse opens devices read-only, counts physical key and mouse-button presses, and does not collect typed text, pointer positions or window titles. Do not run the desktop app with sudo. The collector works independently of X11/Wayland, ignores held-key repeats and reconnects hot-plugged devices.

## Desktop and updates

Close offers hiding to the tray or saving and quitting. Ubuntu's AppIndicator extension must be enabled to show the tray icon. Use Mini to keep a visible counter. Pinning and window placement follow the compositor's capabilities; Wayland may restrict them.

Settings includes launch at login, manual update checks and daily automatic checks. Installing an update requires your action and a writable complete portable folder. Keep the folder at the same path after enabling launch at login.

History and settings live in `$XDG_DATA_HOME/ClickPulse`, normally `~/.local/share/ClickPulse`. Updating or deleting the program folder retains these files. Linux packages use system GTK/WebKit libraries; build the distributed binary on Ubuntu 22.04 before validating it on both supported systems.
