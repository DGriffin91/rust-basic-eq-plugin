# WIP Basic EQ Plugin
A basic EQ vst plugin in rust, using imgui. 

This plugin is primarily a test bed for [rust-basic-audio-filters](https://github.com/DGriffin91/rust-basic-audio-filters)

The plugin logs events to `~/tmp/IMGUIBaseviewEQ.log`.

Until version 1.0, parameters will change and compatibility will not be kept between updates. 


## Usage: macOS (Untested)

- Run `scripts/macos-build-and-install.sh`
- Start your DAW, test the plugin

## Usage: Windows

- Run `cargo build --release`
- Copy `target/release/eq_plugin.dll` to your VST plugin folder
- Start your DAW, test the plugin

## Usage: Linux (Untested)

- Run `cargo build --release`
- Copy `target/release/eq_plugin.so` to your VST plugin folder
- Start your DAW, test the plugin
