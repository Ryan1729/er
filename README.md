This is a template to make it easy to get started with BearLibTerminal.

This has been tested on Linux and Windows. MacOS currently is untested.

## Installing required lib on Linux

This program relies on `libBearLibTerminal.so` so that should be copied into `usr/local/lib` or another folder indicated by this command: `ldconfig -v 2>/dev/null | grep -v ^$'\t'`

then you should run `sudo ldconfig` to complete the installation.

Then the executable should run correctly.

Alternately if your OS has a package for BearLibTerminal, that may work as well.

Once that's done compiling in debug mode with `cargo build` and release mode with `cargo build --release` should work.

## Compiling for Windows

Install [Rust](https://rustup.rs/) if you haven't already.

You will also need a copy of the precompiled `BearLibTerminal.dll` and `BearLibTerminal.lib`. This version is currently tested against 0.15.7 [found here](http://foo.wyrd.name/en:bearlibterminal#download).

`BearLibTerminal.dll` is required to run the program and must be distributed along with the binary.

`BearLibTerminal.lib` is required to build the program.

Copy both `BearLibTerminal.lib` and `BearLibTerminal.dll` to the project root, then building using `cargo` should work as expected.

### Windows release

Run `cargo build --release` then copy the exe in `./target/release` to the desired location as well as `BearLibTerminal.dll` and any necessary assets (graphics, sound, etc.).

### History

This was based on [another template](https://github.com/Ryan1729/live-code-bear-lib-terminal-template) which had a nice feature that was hard to get working cross-platform and which made deploying on windows a pain. This version attempts to increase compatibility by removing the feature.
