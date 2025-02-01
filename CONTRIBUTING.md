# Info for contributors
This is a collection of useful resources for contributors.

## Code structure
- `m64prs`: Emulator and supporting libraries
  - `native`: build scripts for `mupen64plus-core` and supporting plugins
  - `sys`: unsafe bindings to Mupen64Plus and common abstractions for safety
    - API definitions using `decan`
    - Common safe types for errors and config
  - `core`: safe bindings to Mupen64Plus for frontends
  - `plugin-core`: safe bindings to Mupen64Plus for plugins
  - `vcr`: support library for input and media encoding
  - `gtk-utils`: general utilities for working with GTK
  - `gtk-macros`: procedural macros used together with `gtk-utils`
  - `gtk`: The main frontend
- `tasinput`: Input plugin allowing pixel-precise inputs
  - `bridge`: input plugin backend for Mupen64Plus
  - `protocol`: crude socket protocol for IPC between bridge and UI
  - `ui`: UI host, relays input information to bridge over the socket
