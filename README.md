# m64prs
A WIP TASing emulator based on Mupen64Plus and written in Rust. The 

## Building

### Linux
***Dependencies:***
- SDL, SDL_net (v2.x)
- libpng
- FreeType
- zlib
- GTK (at least v4.14)
- `blueprint-compiler`

Use **`./build.py run`** to compile and setup all files.

### Windows
***Dependencies:***
- GTK (at least v4.14, from `gvsbuild`)
- `blueprint-compiler` from `pip`

Use **`./build.py run`** to compile and setup all files. Note that while the codebase
does theoretically support Windows, there are some outstanding bugs in GTK that make it
less than ideal to use. Known bugs include:

- Potential desktop freezes when moving the window ([link](https://gitlab.gnome.org/GNOME/gtk/-/issues/7175)) in mixed-DPI environments

## Currently implemented features
- Callback system on top of Mupen64Plus to hook into input, audio, and savestates
- `.m64` file parser
- Half-working GTK-based UI on Linux (Windows soon(tm))

## To-do list
- Remaining emulator commands
  - Pause, resume, frame advance, reset
  - Saving and loading state
- Key input passthrough
- Video encoding via `rsmpeg`
- VCR features (input recording, savestate linkage)
- Scripting (potentially not via Lua)