use std::env;
use std::ffi::c_int;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sysinfo::Disks;

extern "C" {
    fn growth_add(current: u64, disk_total: u64, result: *mut u64) -> c_int;
}

// Hard safety limits for the on-disk experiment.
// These limits do not change the mathematical recurrence.
const MAX_FILE_BYTES: u64 = 1 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
const WRITE_CHUNK_BYTES: usize = 64 * 1024;
const WRITE_PATTERN: [u8; 16] = *b"SELF-GROWTH-DATA";

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
        "\nTOTAL BASE VALUE: {} bytes ({:.2} GiB)",
        total_bytes,
        total_bytes as f64 / 1_073_741_824.0
    );

    Ok(total_bytes)
}

fn calculate_next(current: u64, disk_total: u64) -> Result<u64, String> {
    let mut next = 0_u64;
    let success = unsafe { growth_add(current, disk_total, &mut next) };

    if success == 1 {
        Ok(next)
    } else {
        Err("The next sum would overflow uint64_t. Calculation stopped safely.".to_string())
    }
}

fn write_cycle_file(
    directory: &Path,
    iteration: u64,
    requested_bytes: u64,
    remaining_budget: u64,
) -> io::Result<u64> {
    if requested_bytes > MAX_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "cycle requests {requested_bytes} bytes; per-file experiment limit is {MAX_FILE_BYTES} bytes"
            ),
        ));
    }

    if requested_bytes > remaining_budget {
        return Err(io::Error::new(
            io::ErrorKind::StorageFull,
            format!(
                "cycle requests {requested_bytes} bytes; only {remaining_budget} bytes remain in the experiment budget"
            ),
        ));
    }

    let file_path: PathBuf = directory.join(format!("cycle-{iteration:08}.bin"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)?;

    let mut remaining = requested_bytes;
    let mut buffer = [0_u8; WRITE_CHUNK_BYTES];

    for (index, byte) in buffer.iter_mut().enumerate() {
        *byte = WRITE_PATTERN[index % WRITE_PATTERN.len()];
    }

    while remaining > 0 {
        let amount = remaining.min(buffer.len() as u64) as usize;
        file.write_all(&buffer[..amount])?;
        remaining -= amount as u64;
    }

    file.sync_all()?;

    let actual_bytes = fs::metadata(&file_path)?.len();
    if actual_bytes != requested_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file size verification failed: requested {requested_bytes}, actual {actual_bytes}"
            ),
        ));
    }

    Ok(actual_bytes)
}

fn main() {
    println!("======================================");
    println!("          SELF-GROWTH.EXE");
    println!("     RECURSIVE CALCULATION + WRITER");
    println!("======================================");

    let scanned_total = match scan_all_disks() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[SCAN ERROR] {error}");
            return;
        }
    };

    // Optional small seed for controlled lab runs.
    // Example (PowerShell): $env:SELF_GROWTH_TEST_BASE_BYTES="1024"
    let (disk_total, test_override) = match env::var("SELF_GROWTH_TEST_BASE_BYTES") {
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
            "\n[LAB OVERRIDE] Using test base: {disk_total} bytes. Scanned capacity remains informational."
        );
    }

    let output_dir = Path::new("self-growth-lab");
    if let Err(error) = fs::create_dir_all(output_dir) {
        eprintln!("[WRITER ERROR] Cannot create lab directory: {error}");
        return;
    }

    let mut accumulated = disk_total;
    let mut iteration: u64 = 0;
    let mut total_written: u64 = 0;

    println!("\n===== CALCULATION + DISK WRITING =====");
    println!("S(0) = T = {accumulated} bytes");
    println!("Output directory: {}", output_dir.display());
    println!("Maximum file size: {MAX_FILE_BYTES} bytes");
    println!("Maximum total written: {MAX_TOTAL_BYTES} bytes");
    println!("Interval: 2 seconds");
    println!("Press Ctrl+C to stop.");

    loop {
        match write_cycle_file(
            output_dir,
            iteration,
            accumulated,
            MAX_TOTAL_BYTES - total_written,
        ) {
            Ok(written) => {
                total_written += written;
                println!(
                    "[WRITE OK] Cycle {iteration} | S({iteration}) = {accumulated} bytes | file written = {written} bytes | total written = {total_written} bytes"
                );
            }
            Err(error) => {
                eprintln!(
                    "[WRITER STOP] Cycle {iteration} was not written: {error}"
                );
                break;
            }
        }

        thread::sleep(Duration::from_secs(2));

        match calculate_next(accumulated, disk_total) {
            Ok(next) => {
                accumulated = next;
                iteration = match iteration.checked_add(1) {
                    Some(value) => value,
                    None => {
                        eprintln!("[CALCULATION STOP] Iteration counter overflow.");
                        break;
                    }
                };
                println!(
                    "[CALCULATION] Next value S({iteration}) = {accumulated} bytes"
                );
            }
            Err(error) => {
                eprintln!("[CALCULATION STOP] {error}");
                eprintln!("Last valid accumulated value: {accumulated} bytes");
                break;
            }
        }
    }

    println!(
        "\n[FINISHED] Cycles written: {} | Total physical payload written: {} bytes",
        iteration.saturating_add(1),
        total_written
    );
}
