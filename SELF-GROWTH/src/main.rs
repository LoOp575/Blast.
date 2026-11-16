use std::ffi::c_int;
use std::thread;
use std::time::Duration;

use sysinfo::Disks;

extern "C" {
    fn growth_add(current: u64, disk_total: u64, result: *mut u64) -> c_int;
}

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

fn main() {
    println!("======================================");
    println!("          SELF-GROWTH.EXE");
    println!("          DISK SUMMATION V2");
    println!("======================================");

    // Discover disk capacities once. This fixed total is the summation increment.
    let disk_total = match scan_all_disks() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[SCAN ERROR] {error}");
            return;
        }
    };

    // S(0) = T, where T is the sum of all detected disk capacities.
    let mut accumulated = disk_total;
    let mut iteration: u64 = 0;

    println!("\n===== CONTINUOUS SUMMATION =====");
    println!("S(0) = T = {accumulated} bytes");
    println!("Interval: 2 seconds");
    println!("Press Ctrl+C to stop.");

    loop {
        thread::sleep(Duration::from_secs(2));

        match calculate_next(accumulated, disk_total) {
            Ok(next) => {
                accumulated = next;
                iteration = iteration.saturating_add(1);

                println!(
                    "Cycle {iteration} | S({iteration}) = {} bytes ({:.2} GiB)",
                    accumulated,
                    accumulated as f64 / 1_073_741_824.0
                );
            }
            Err(error) => {
                eprintln!("[CALCULATION STOP] {error}");
                eprintln!("Last valid accumulated value: {accumulated} bytes");
                break;
            }
        }
    }
}
