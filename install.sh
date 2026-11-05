#!/usr/bin/env bash
# rustpaper on Linux, in one command:
#
#   curl -fsSL https://raw.githubusercontent.com/SaphirePay/rustpaper/main/install.sh | bash
#
# It installs what building needs (your distribution's packages, Rust), downloads this repo, the Horizon
# server runtime and a Java runtime (only used to prepare the game files), builds the Paper engine and the
# server panel, and prepares the game files from your own Minecraft if it is installed. Nothing of
# Minecraft is downloaded: the game files are always made from your own copy.
#
# Settings (environment variables):
#   RUSTPAPER_DIR=PATH        where it goes (default: ~/rustpaper)
#   RUSTPAPER_NO_PACKAGES=1   don't install system packages (you have them, or no sudo)
#   RUSTPAPER_NO_PANEL=1      skip the panel (a server with no desktop)
#   MINECRAFT_DIR=PATH        your Minecraft folder, if it isn't ~/.minecraft
#   RUSTPAPER_SOURCE=URL      clone from another copy of the repo (a fork, or a local folder)
set -euo pipefail

say() { printf '\033[1;35m==>\033[0m %s\n' "$*"; }
note() { printf '    %s\n' "$*"; }
fail() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

[ "$(uname -s)" = Linux ] || fail "this installer is for Linux (on Windows, follow the README)"
[ "$(uname -m)" = x86_64 ] || fail "the Horizon server runtime is published for x86_64 Linux only"

DIR="${RUSTPAPER_DIR:-$HOME/rustpaper}"
MC="${MINECRAFT_DIR:-$HOME/.minecraft}"
SUDO=""
if [ "$(id -u)" -ne 0 ]; then SUDO="sudo"; fi

# 1. What building and the panel need from the system.
if [ "${RUSTPAPER_NO_PACKAGES:-}" != 1 ]; then
  say "Installing build tools (your password may be asked for)"
  if command -v apt-get >/dev/null; then
    $SUDO apt-get update -qq
    $SUDO apt-get install -y -qq git curl ca-certificates python3 build-essential pkg-config \
      libasound2-dev libudev-dev libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 \
      libx11-6 libx11-xcb1 libxcursor1 libxrandr2 libxi6 >/dev/null
  elif command -v dnf >/dev/null; then
    $SUDO dnf install -y -q git curl python3 gcc gcc-c++ make pkgconf-pkg-config alsa-lib-devel \
      systemd-devel libxkbcommon-x11 mesa-libGL mesa-libEGL wayland-devel libX11 libXcursor libXrandr libXi
  elif command -v pacman >/dev/null; then
    $SUDO pacman -S --needed --noconfirm git curl python base-devel pkgconf alsa-lib systemd-libs \
      libxkbcommon-x11 mesa wayland libx11 libxcursor libxrandr libxi
  elif command -v zypper >/dev/null; then
    $SUDO zypper --non-interactive install git curl python3 gcc gcc-c++ make pkg-config alsa-devel \
      libudev-devel libxkbcommon-x11-0 Mesa-libGL1 Mesa-libEGL1 libwayland-client0 libX11-6 libXcursor1 \
      libXrandr2 libXi6
  else
    note "Unknown package manager: install git, curl, python3, a C compiler, pkg-config and the"
    note "ALSA and udev development files yourself, then run this again with RUSTPAPER_NO_PACKAGES=1."
    exit 1
  fi
fi
for tool in git curl python3 cc; do
  command -v "$tool" >/dev/null || fail "$tool is missing"
done

# 2. Rust (rustup; setup.py installs the exact toolchain pins.json names).
if ! command -v cargo >/dev/null; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    say "Installing Rust (rustup)"
    # rustup puts ~/.cargo/bin on your PATH for new terminals (your shell's profile).
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain none --profile minimal >/dev/null
    export PATH="$HOME/.cargo/bin:$PATH"
  fi
fi

# 3. This repo.
if [ -d "$DIR/.git" ]; then
  say "Updating $DIR"
  git -C "$DIR" pull --ff-only -q
else
  say "Downloading rustpaper into $DIR"
  git clone -q --depth 1 "${RUSTPAPER_SOURCE:-https://github.com/SaphirePay/rustpaper.git}" "$DIR"
fi
cd "$DIR"

# 4. The runtime, Java (for preparing only), the engine and the panel.
say "Downloading the Horizon server runtime"
python3 setup.py runtime
say "Downloading a Java runtime (used only to prepare the game files)"
python3 setup.py java
say "Building the Paper engine (about 5 to 15 minutes the first time)"
python3 setup.py engine
if [ "${RUSTPAPER_NO_PANEL:-}" != 1 ]; then
  say "Building the server panel (about 2 minutes)"
  python3 setup.py panel --build --no-open
fi

# 5. The game files, from your own Minecraft.
prepared=0
if [ -d "$MC/versions/26.3" ]; then
  say "Preparing the game files from your Minecraft ($MC)"
  python3 setup.py prepare --minecraft-dir "$MC" && prepared=1
fi

echo
say "rustpaper is installed in $DIR"
if [ "$prepared" = 1 ]; then
  note "Make a server:   cd $DIR && python3 setup.py server myserver --offline --op YourName"
  note "Start it:        python3 setup.py start myserver          (opens the panel)"
  note "Headless:        python3 setup.py start myserver --nogui"
else
  note "One step left: the game files are made from your own copy of Minecraft Java Edition 26.3."
  note "  - On this computer: install the official Minecraft launcher, play 26.3 once, then run"
  note "      cd $DIR && python3 setup.py prepare"
  note "  - A server with no desktop: prepare on a computer that has Minecraft (same repo, Windows"
  note "    or Linux), then copy its runtime/.local/horizon-client/26.3 folder to"
  note "    $DIR/runtime/.local/horizon-client/26.3 here."
  note "Then: python3 setup.py server myserver --offline --op YourName && python3 setup.py start myserver"
fi
note "Everything else: python3 setup.py doctor, and docs/SERVER.md, docs/PANEL.md"
note "(Rust is on the PATH in new terminals; in this one: source ~/.cargo/env)"
