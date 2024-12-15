# m64prs
A WIP TASing emulator based on Mupen64Plus and written in Rust. The 

## Building

### Linux
***Dependencies:***
- SDL, SDL_net (v2.x)
- libpng
- FreeType
- zlib
- GTK 4 (at least 4.14)
- Python (at least 3.10)
- `blueprint-compiler`

Use **`./build.py run`** to compile and setup all files.

### Windows
As of now, I currently recommend compiling and running `m64prs` under WSL. Most platform code should be present on Windows,
but GTK is quite buggy. See the below links for more information.
- [GTK issue !7175](https://gitlab.gnome.org/GNOME/gtk/-/issues/7175) &ndash; GTK partially freezes the desktop under
  specific circumstances in a mixed-DPI setup
- [GTK issue !1036](https://gitlab.gnome.org/GNOME/gtk/-/issues/1036) &ndash; Support fractional scaling on Windows

#### Setting up GTK4 and `blueprint-compiler`
First things first:

- Install Visual Studio 2022 *with a Windows 10 SDK* (preferably the latest one).
- Install the latest stable Rust compiler through *rustup*.
- Install Python ***SPECIFICALLY*** through the Microsoft Store, due to [a bug with GTK's build tooling](https://github.com/wingtk/gvsbuild/pull/1474).
  - If you installed Python via `winget` or the installer from the website, it *may not work* unless they fixed it.

Install GTK and some Python dependencies:
```powershell
# Install gvsbuild
pip install gvsbuild pyinstaller
gvsbuild build --py-wheel --enable-gi gtk4 pygobject
```

In a separate directory, clone and package `blueprint-compiler`:
```powershell
# You don't need to keep this directory around anyways.
git clone --depth 1 -b v0.14.0 "https://gitlab.gnome.org/jwestman/blueprint-compiler.git"
cd blueprint-compiler
pyinstaller blueprint-compiler.py
```

Move the `pyinstaller` package generated in the `dist` folder to wherever you install
local software. Add its folder to your `PATH` as well, since gtk-rs needs it there.

> If you can find a way to avoid pyinstaller, please let me know or PR changes to these instructions.
> You'll be saving some headache.

#### Native Build
***Dependencies:***
- GTK (at least v4.14, from `gvsbuild`)
- `blueprint-compiler`

Use **`./build.py run`** to compile and setup all files. Due to the aforementioned bugs, Windows support will be limited
until those bugs are fixed.

## Currently implemented features
- Callback system on top of Mupen64Plus to hook into input, audio, and savestates
- `.m64` file parser
- Half-working GTK-based UI on Linux (Windows soon&trade;)

## To-do list
- Remaining emulator commands
  - Pause, resume, frame advance, reset
  - Saving and loading state
- Key input passthrough
- Video encoding via `rsmpeg`
- VCR features (input recording, savestate linkage)
- Scripting (potentially not via Lua)