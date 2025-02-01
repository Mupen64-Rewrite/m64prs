# m64prs
A WIP TASing emulator based on Mupen64Plus and written in Rust. This is my attempt to clean up some of the rougher points
of [M64RPFW](https://github.com/Mupen64-Rewrite/M64RPFW) and simplify the core integration so that we can keep up with
features and fixes added to Mupen64Plus.

## License
This project is licensed under the GNU General Public License, version 3.0 or later. See `LICENSE.md` for details.

## Building
MSRV (**M**inimum **S**upported **R**ust **V**ersion) is currently the latest stable release of Rust.

### Linux
***Dependencies:***
- SDL, SDL_net (v2.x)
- libpng
- FreeType
- zlib
- GTK 4 (at least 4.14)
- Python (at least 3.10)

Use **`./build.py run`** to compile and setup all files.

### Windows
As of now, I currently recommend compiling and running `m64prs` under WSL, particularly if you're working with HiDPI
screens. I'd like to implement HiDPI properly, though GTK has yet to support fractional scaling on Windows.
- [GTK issue !7175](https://gitlab.gnome.org/GNOME/gtk/-/issues/7175) &ndash; GTK partially freezes the desktop under
  specific circumstances in a mixed-DPI setup
- [GTK issue !1036](https://gitlab.gnome.org/GNOME/gtk/-/issues/1036) &ndash; Support fractional scaling on Windows

#### Setting up GTK4
First things first:

- Install Visual Studio 2022 *with a Windows 10 SDK* (preferably the latest one).
- Install the latest stable Rust compiler through *rustup*.
- Install Python ***SPECIFICALLY*** through the Microsoft Store, due to [a bug with GTK's build tooling](https://github.com/wingtk/gvsbuild/pull/1474).
  - If you installed Python via `winget` or the installer from the website, it *may not work* unless they fixed it. (UPDATE: they fixed it, but I haven't tested myself.)

Install GTK:
```powershell
# Install gvsbuild
pip install gvsbuild
gvsbuild build gtk4
```

Use **`./build.py run`** to compile and setup all files. Due to the aforementioned bugs, 
Windows support will be limited until those bugs are fixed.

## Currently implemented features
- Callback system on top of Mupen64Plus to hook into input, audio, and savestates
- `.m64` file parser
- GTK-based UI on Linux and Windows (half-broken ATM)
- VCR features (input recording, savestate linkage)
- An equivalent to TASInput
- Key input passthrough

## To-do list
- Video encoding via `rsmpeg`
- Scripting (potentially not via Lua)