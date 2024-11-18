# m64prs
A WIP TASing emulator based on Mupen64Plus and written in Rust. This project is currently waiting on
[`winit`](https://github.com/rust-windowing/winit) to add a few features (popups and child windows).
Beyond that, it is mostly working.

## Building
***Dependencies (Linux, MacOS):***
- SDL, SDL_net (v2.x)
- libpng
- FreeType
- zlib

Use **`./build.py run`** to compile and setup all files in the install directory.

***DO NOT*** run `cargo run` directly, as the executable locates its libraries and data files
using paths relative to it self.

## Currently implemented features
- Callback system on top of Mupen64Plus to hook into input, audio, and savestates
- `.m64` file parser
- Half-working GTK-based UI on Linux (Windows soon(tm))

## To-do list
- Video encoding via `rsmpeg`
- VCR features (input recording, savestate linkage)
- Scripting (potentially not via Lua)