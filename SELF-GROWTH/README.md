# SELF-GROWTH.EXE

Initial C + Rust implementation.

## Structure

- `src/recursive.c`: recursive calculation engine.
- `src/main.rs`: self-discovery, all-disk scan, and continuous controller.
- `build.rs`: compiles the C engine through the `cc` crate.

## Formula

```
x(n+1) = x(n)^2 + 1
```

Initial independent values:

- MEMORY = 1
- ENGINE = 2
- GROWTH = 3

Each active value is advanced once per controller loop. The loop repeats every two seconds.

## Build

Install Rust and a compatible C compiler, then run from this directory:

```sh
cargo build --release
cargo run --release
```

On Windows, the release executable is `target/release/self-growth.exe`.

## Notes

- The C engine uses checked `uint64_t` arithmetic. A value stops advancing if its next result would overflow; the Rust controller continues.
- Disk scanning is read-only. The application does not fill disks or impose an application storage quota.
- Stop the process with Ctrl+C in the terminal.
- This source has not yet been compiled or runtime-tested in this repository environment.
