# Building

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- [gtk4](https://www.gtk.org/docs/installations/)

## compiling
```sh
cargo build
```

## Warning
If you encounter issues with gtk4 being "missing" despite installing it, you may need to install the `libgtk-4-dev` package.
If that still doesn't work, you need to find the gtk4.pc file with:
```sh
sudo find / -name gtk4.pc 2>/dev/null
```
Then set the PKG_CONFIG_PATH environment variable to the directory containing the gtk4.pc file:
```sh
export PKG_CONFIG_PATH=/path/to/gtk4.pc
```
Then try to compile again.
