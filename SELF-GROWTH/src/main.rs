use num_bigint::BigUint;
use std::env;
use std::time::Instant;
use sysinfo::Disks;

// Memory-I/O experiment budget. This bounds the amount of byte work per run,
// not the mathematical representation of the recurrence.
const MAX_TEST_BYTES: u64 = 64 * 1024 * 1024;
const CHUNK_BYTES: usize = 64 * 1024;
const BYTE_PATTERN: u8 = 0xAA;

fn scan_all_disks() -> Result<u64, String> {
    let disks = Disks::new_with_refreshed_list();
    let mut total_bytes = 0_u64;

    println!("\n===== INITIAL DISK SCAN =====");

    if disks.list().is_empty() {
        return Err("No mounted disks or volumes were detected.".to_string());
    }

    for disk in disks.list() {
        let capacity = disk.total_space();
        total_bytes = total_bytes
            .checked_add(capacity)
            .ok_or_else(|| "Total disk capacity exceeds the u64 range.".to_string())?;

        println!(
            "Volume: {:?} | Capacity: {} bytes ({:.2} GiB)",
            disk.mount_point(),
            capacity,
            capacity as f64 / 1_073_741_824.0
        );
    }

    println!(
        "\nTOTAL DISK BASE: {} bytes ({:.2} GiB)",
        total_bytes,
        total_bytes as f64 / 1_073_741_824.0
    );

    Ok(total_bytes)
}

fn process_memory_bytes(target_bytes: u64, buffer: &mut [u8]) -> u64 {
    let mut processed = 0_u64;

    while processed < target_bytes {
        let amount = (target_bytes - processed).min(buffer.len() as u64) as usize;

        // Memory I/O: overwrite a bounded RAM buffer with the output pattern.
        buffer[..amount].fill(BYTE_PATTERN);
        std::hint::black_box(&buffer[..amount]);

        processed += amount as u64;
    }

    processed
}

fn main() {
    println!("======================================");
    println!("       SELF-GROWTH MEMORY ENGINE");
    println!("     RECURSIVE VALUE + MEMORY I/O");
    println!("======================================");

    let scanned_total = match scan_all_disks() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[SCAN ERROR] {error}");
            return;
        }
    };

    // Optional small seed for controlled tests while retaining the real scan.
    // PowerShell: $env:SELF_GROWTH_TEST_BASE_BYTES="1024"
    let (base_value, test_override) = match env::var("SELF_GROWTH_TEST_BASE_BYTES") {
        Ok(value) => match value.parse::<u64>() {
            Ok(parsed) if parsed > 0 => (parsed, true),
            _ => {
                eprintln!("[CONFIG ERROR] SELF_GROWTH_TEST_BASE_BYTES must be a positive integer.");
                return;
            }
        },
        Err(_) => (scanned_total, false),
    };

    if test_override {
        println!(
            "[LAB OVERRIDE] Using {base_value} bytes as the recurrence seed. Disk scan remains informational."
        );
    }

    println!("Memory-I/O test budget: {MAX_TEST_BYTES} bytes");
    println!("Memory-I/O chunk size: {CHUNK_BYTES} bytes");
    println!("Byte pattern: 0x{BYTE_PATTERN:02X}");
    println!("No files are created and no disk data is written.");
    println!("Press Ctrl+C to stop.");

    let mut current = BigUint::from(base_value);
    let test_budget = BigUint::from(MAX_TEST_BYTES);
    let mut buffer = vec![0_u8; CHUNK_BYTES];
    let mut cycle = 0_u64;
    let mut total_processed = 0_u64;
    let run_started = Instant::now();

    println!("\n===== RECURSIVE MEMORY-I/O TEST =====");

    loop {
        // The numeric value is also the requested byte-work amount.
        // Keep the arbitrary-precision value, but only process values within
        // the explicit lab budget so the test cannot exhaust system RAM.
        if current > test_budget {
            println!(
                "[TEST STOP] Next target exceeds the {MAX_TEST_BYTES}-byte lab budget."
            );
            break;
        }

        let target_bytes = current
            .to_u64_digits()
            .first()
            .copied()
            .unwrap_or(0);

        let started = Instant::now();
        let processed = process_memory_bytes(target_bytes, &mut buffer);
        let elapsed = started.elapsed();

        total_processed = match total_processed.checked_add(processed) {
            Some(value) => value,
            None => {
                eprintln!("[TEST STOP] Processed-byte counter overflow.");
                break;
            }
        };

        println!(
            "[MEMORY I/O] Cycle {cycle} | numeric_value = {current} bytes | processed = {processed} bytes | time = {:.3} ms",
            elapsed.as_secs_f64() * 1000.0
        );

        // Original self-summing recurrence: S(n+1) = S(n) + S(n).
        current = &current + &current;

        cycle = match cycle.checked_add(1) {
            Some(value) => value,
            None => {
                eprintln!("[TEST STOP] Cycle counter overflow.");
                break;
            }
        };
    }

    println!(
        "\n===== PERFORMANCE SUMMARY =====\nCycles processed: {cycle}\nTotal byte-work: {total_processed} bytes\nTotal elapsed: {:.3} seconds\nRAM working buffer: {} bytes",
        run_started.elapsed().as_secs_f64(),
        buffer.len()
    );
    println!(
        "Note: byte-work is processed in a reusable fixed-size RAM buffer; the entire numeric target is not allocated at once."
    );
}
