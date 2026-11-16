# SELF-GROWTH.EXE

Rust memory-I/O experiment with a C calculation source retained in the project.

## Core behavior

SELF-GROWTH scans mounted volumes and sums their reported total capacities. That total becomes the initial numeric seed. The engine repeatedly adds the current value to itself:

```
T = D1 + D2 + ... + Dn
S(0) = T
S(n+1) = S(n) + S(n)
```

The numeric result is treated as a byte-work target. The program processes that many bytes through a reusable RAM buffer filled with the byte pattern `0xAA`.

## Memory-I/O behavior

- No output files are created.
- No disk data is written.
- The numeric recurrence uses `num-bigint::BigUint`.
- There is no preset total byte-work limit.
- Byte processing uses a reusable 64 KiB RAM chunk. The chunk is reused repeatedly until the complete target byte count has been processed.
- The entire numeric target is not allocated in RAM at once.
- Press Ctrl+C to stop a long-running test.

The 64 KiB chunk is a per-operation buffer size, not a limit on the total number of bytes processed. Keeping it bounded prevents a large recurrence value from being allocated as one enormous RAM block.

The scanner reports mounted-volume capacities. Depending on the operating system, partitions or mount points may represent the same physical device and can be counted separately.

## Controlled test

A real machine's total disk capacity can produce a very large first byte-work target. Use a small seed to verify the recurrence and chunked processing first:

PowerShell:

```powershell
$env:SELF_GROWTH_TEST_BASE_BYTES="1024"
cargo run --release
```

This still performs and displays the disk scan, but uses 1024 bytes as the recurrence seed for the controlled test. The byte-work targets begin at 1024, 2048, 4096, 8192, and so on. Since there is no total-work limit, stop the run with Ctrl+C when the test is sufficient.

Remove the override:

```powershell
Remove-Item Env:SELF_GROWTH_TEST_BASE_BYTES
```

## Architecture

- `src/main.rs` (Rust): scans mounted volumes, controls arbitrary-precision recurrence, and performs chunked memory-buffer byte processing.
- `src/recursive.c` (C): retained checked 64-bit addition implementation; the current BigUint memory experiment does not call this C function.
- `build.rs`: builds the retained C source through the `cc` crate.

## Important technical notes

- The recurrence value can grow beyond 64-bit integer range because it uses `BigUint`.
- Total byte-work has no preset cap, so large targets can take an impractically long time to finish.
- The 64 KiB buffer is reused. A numeric value of 1 GiB therefore represents 1 GiB of byte-work, not a 1 GiB resident RAM allocation.
- Actual memory usage stays bounded by the reusable buffer and the size of the BigUint value.
- This is an unbounded-work experiment, not an unbounded RAM-allocation experiment. A very large target can consume substantial CPU time even though the RAM buffer remains fixed.

## Build and run

Install Rust and a compatible C compiler, then run from this directory:

```powershell
cargo build --release
cargo run --release
```

Compilation and runtime testing must still be performed on the target Windows machine.
