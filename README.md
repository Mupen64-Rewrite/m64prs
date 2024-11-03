# m64prs
A WIP TASing emulator based on Mupen64Plus and written in Rust. This project is currently waiting on
[`winit`](https://github.com/rust-windowing/winit) to add a few features (popups and child windows).
Beyond that, it is mostly working.

## Currently implemented features
- Callback system on top of Mupen64Plus to hook into input, audio, and savestates
- `.m64` file parser
- Command-line winit-based UI for testing (see `m64prs-testbed`)

## To-do list
- Video encoding via `rsmpeg`
- Actual GUI (blocked on `winit`)