# usbredirparser-rs

A Rust implementation of the `usbredirparser` C library, maintaining ABI compatibility.

I (Titus Rwantare) am taking on this project to learn C/Rust interop.

## Warning

Extensive use of unsafe and LLM generated tests. This code may set your computer on fire.

## Credits

This project is a port of the original usbredir parser at
http://gitlab.freedesktop.org/spice/usbredir

The original library is licensed under LGPL-2.1 or later.

## Status

Currently implemented:
*   `usbredirparser_create`
*   `usbredirparser_init`
*   `usbredirparser_destroy`

## Build

This project uses `cbindgen` to automatically generate C headers.

```bash
cargo build
```

The generated header can be found at `include/usbredirparser.h`.
