use num_bigint::BigUint;
use std::env;
use std::time::Instant;
use sysinfo::Disks;

// LAB_UNBOUNDED removes the total byte-work budget.
// This is a reusable working-buffer size, not the total amount processed.
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

// Processes exactly target_bytes bytes of work by repeatedly reusing
// one fixed-size RAM buffer. No file is created or written.
fn process_memory_bytes(target_bytes: &BigUint, buffer: &mut [u8]) -> BigUint {
    let mut remaining = target_bytes.clone();
    let mut processed = BigUint::from(0_u8);
    let chunk_size = BigUint::from(buffer.len());

    while remaining > BigUint::from(0_u8) {
        let amount = if remaining >= chunk_size {
            buffer.len()
        } else {
            // remaining < buffer.len(), so its low u64 limb is sufficient.
            remaining.to_u64_digits().first().copied().unwrap_or(0) as usize
        };

        if amount == 0 {
            break;
        }

        buffer[..amount].fill(BYTE_PATTERN);
        std::hint::black_box(&buffer[..amount]);

        let completed = BigUint::from(amount);
        processed += &completed;
        remaining -= completed;
    }

    processed
}

fn main() {
    println!("======================================");
    println!("          SELF-GROWTH.EXE");
    println!("          LAB_UNBOUNDED MODE");
    println!("======================================");

    let scanned_total = match scan_all_disks() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[SCAN ERROR] {error}");
            return;
        }
    };

    // Optional controlled seed for lab experiments.
    // PowerShell: $env:SELF_GROWTH_TEST_BASE_BYTES="1024"
    let (base_value, test_override) = match env::var("SELF_GROWTH_TEST_BASE_BYTES") {
        Ok(value) => match value.parse::<u64>() {
            Ok(parsed) if parsed > 0 => (parsed, true),
            _ => {
                eprintln!(
                    "[CONFIG ERROR] SELF_GROWTH_TEST_BASE_BYTES must be a positive integer."
                );
                return;
            }
        },
        Err(_) => (scanned_total, false),
    };

    if test_override {
        println!(
            "[LAB OVERRIDE] Using {base_value} bytes as recurrence seed. Disk scan remains informational."
        );
    }

    println!("Total byte-work budget: NONE");
    println!("Recursive integer: arbitrary precision (BigUint)");
    println!("RAM working chunk: {CHUNK_BYTES} bytes, reused continuously");
    println!("Byte pattern: 0x{BYTE_PATTERN:02X}");
    println!("Disk writes: DISABLED");
    println!("Stop the process with Ctrl+C.");
    println!(
        "Important: byte-work is uncapped, but the RAM working buffer is fixed-size. Very large targets can take an impractically long time."
    );

    let mut current = BigUint::from(base_value);
    let mut buffer = vec![0_u8; CHUNK_BYTES];
    let mut cycle = 0_u64;
    let mut total_processed = BigUint::from(0_u8);
    let run_started = Instant::now();

    loop {
        println!("\n===== CYCLE {cycle} =====");
        println!("Recursive target: {current} bytes");

        let started = Instant::now();
        let processed = process_memory_bytes(&current, &mut buffer);
        let elapsed = started.elapsed();

        total_processed += &processed;

        println!("Processed: {processed} bytes");
        println!("Processing time: {:.3} seconds", elapsed.as_secs_f64());

        // S(n+1) = S(n) + S(n)
        current = &current + &current;

        cycle = match cycle.checked_add(1) {
            Some(value) => value,
            None => {
                eprintln!("[STOP] Cycle counter overflow.");
                break;
            }
        };
    }

    println!("\n===== LAB SUMMARY =====");
    println!("Completed cycles: {cycle}");
    println!("Total byte-work: {total_processed} bytes");
    println!("Total elapsed: {:.3} seconds", run_started.elapsed().as_secs_f64());
    println!("RAM working chunk: {} bytes", buffer.len());
}
