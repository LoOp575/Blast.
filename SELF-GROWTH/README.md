# SELF-GROWTH.EXE

C + Rust implementation: **Disk Core Discovery and Continuous Summation**.

## Core behavior

SELF-GROWTH does not search for or evaluate installed applications. It discovers mounted disks/volumes, reads each volume's total capacity, sums those capacities once, and then continuously adds that fixed baseline to the accumulated value.

## Formula

Let each detected volume have total capacity (D_i).

```
T = D1 + D2 + ... + Dn
S(0) = T
S(k+1) = S(k) + T
```

Example, if the detected capacities total 2 TB:

- Initial value S(0): 2 TB
- Cycle 1: 4 TB
- Cycle 2: 6 TB
- Cycle 3: 8 TB

The displayed accumulated value is a mathematical counter. It is **not** physical disk usage and does not write or allocate data on any disk.

## Architecture

- `src/main.rs` (Rust): discovers mounted volumes once, reads total capacity, establishes the baseline, and controls the continuous loop.
- `src/recursive.c` (C): performs checked addition for each summation cycle.
- `build.rs`: compiles the C calculation engine through the `cc` crate.

## Runtime behavior

- Disk discovery occurs once at startup.
- Every detected mounted volume contributes its reported total capacity to the baseline.
- The controller adds the same baseline every two seconds.
- Ctrl+C stops the process normally.
- If the result exceeds `uint64_t`, calculation stops safely and reports the last valid value.
- No files are modified, no disk space is allocated, and no application storage quota is imposed.

Note: the scanner enumerates mounted volumes exposed by the operating system. Separate partitions or mount points are counted as separate volumes. This is not guaranteed to represent unique physical drives.

## Build

Install Rust and a compatible C compiler, then run from this directory:

```sh
cargo build --release
cargo run --release
```

On Windows, the release executable is `target/release/self-growth.exe`.

## Verification status

Source files were updated in the repository. Compilation and runtime testing have not yet been performed in this environment.
