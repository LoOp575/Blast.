# SELF-GROWTH.EXE

C + Rust implementation: **Recursive Calculation + Disk Writer**.

## Core behavior

SELF-GROWTH discovers mounted volumes and sums their reported capacities once. The sum becomes a fixed recurrence increment. Each recurrence value is then used as the exact requested size of a new data file in the local `self-growth-lab` directory.

## Formula

```
T = D1 + D2 + ... + Dn
S(0) = T
S(k+1) = S(k) + T
```

For each cycle k, the writer creates a new file:

```
cycle-00000000.bin  -> S(0) bytes
cycle-00000001.bin  -> S(1) bytes
cycle-00000002.bin  -> S(2) bytes
...
```

Each file is filled with a repeated deterministic byte pattern. The program verifies the resulting file length after writing.

## Safety limits

The writer is deliberately restricted to a local experiment directory:
- Maximum individual file: 1 MiB.
- Maximum combined payload written by one run: 16 MiB.
- The writer stops before creating a file that would exceed either limit.
- It does not write to every discovered disk, alter existing files, or attempt to fill free disk space.
- These storage limits do not alter the recurrence calculation.

Because real disk capacity totals are normally much larger than 1 MiB, a normal scan may cause the writer to stop before its first file. Use the explicit test override for small controlled tests.

## Controlled test

PowerShell:

```powershell
$env:SELF_GROWTH_TEST_BASE_BYTES="1024"
cargo run --release
```

This keeps the disk scan visible but uses 1024 as the recurrence seed for the lab run. Expected file sizes begin at 1024, 2048, 3072, 4096 bytes, and so on. The run stops automatically at the configured per-file or total-write limit.

Remove the override in PowerShell with:

```powershell
Remove-Item Env:SELF_GROWTH_TEST_BASE_BYTES
```

## Architecture

- `src/main.rs` (Rust): scans mounted volumes, controls the recurrence, writes one new file per cycle, and verifies each file size.
- `src/recursive.c` (C): performs checked 64-bit addition.
- `build.rs`: compiles the C calculation engine through the `cc` crate.

## Important technical limits

- The current recurrence uses unsigned 64-bit integers and stops safely before overflow. It is not mathematically unbounded.
- The scanner enumerates mounted volumes exposed by the operating system. Partitions and mount points may be counted separately; this does not guarantee unique physical-drive capacity.
- The writer writes to the filesystem containing the program's current working directory, not automatically to every scanned volume.

## Build and run

Install Rust and a compatible C compiler, then run from this directory:

```powershell
cargo build --release
cargo run --release
```

Compilation and runtime testing must still be performed on the target Windows machine.
