use std::ffi::c_int;
use std::process;
use std::thread;
use std::time::Duration;

use sysinfo::{Disks, System};

extern "C" {
    fn recursive_step(x: u64, result: *mut u64) -> c_int;
}

struct GrowthValue {
    name: &'static str,
    value: u64,
    active: bool,
}

impl GrowthValue {
    fn new(name: &'static str, initial: u64) -> Self {
        Self {
            name,
            value: initial,
            active: true,
        }
    }

    fn calculate(&mut self) {
        if !self.active {
            return;
        }

        let mut result = 0_u64;
        let success = unsafe { recursive_step(self.value, &mut result) };

        if success == 1 {
            self.value = result;
        } else {
            self.active = false;
            println!("[OVERFLOW] {} reached the u64 limit; value frozen.", self.name);
        }
    }
}

fn scan_disks() {
    let disks = Disks::new_with_refreshed_list();

    println!("\n===== DISK SCANNER =====");

    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);

        let status = if available == 0 {
            "FULL"
        } else {
            "AVAILABLE"
        };

        println!(
            "Disk: {:?} | Total: {} MB | Used: {} MB | Free: {} MB | {}",
            disk.mount_point(),
            total / 1_048_576,
            used / 1_048_576,
            available / 1_048_576,
            status
        );
    }
}

fn self_discovery(system: &System) {
    let pid = process::id();

    println!("\n===== SELF DISCOVERY =====");
    println!("Process ID: {}", pid);

    if let Some(current) = system.process(sysinfo::Pid::from_u32(pid)) {
        println!("Process RAM: {} MB", current.memory() / 1_048_576);
    } else {
        println!("Process information unavailable");
    }
}

fn main() {
    println!("==============================");
    println!("      SELF-GROWTH.EXE");
    println!("      VERSION 1.0");
    println!("==============================");

    let mut memory = GrowthValue::new("MEMORY", 1);
    let mut engine = GrowthValue::new("ENGINE", 2);
    let mut growth = GrowthValue::new("GROWTH", 3);

    let mut system = System::new_all();
    let mut iteration: u64 = 0;

    loop {
        iteration = iteration.saturating_add(1);
        println!("\n========== LOOP {} ==========", iteration);

        system.refresh_all();
        self_discovery(&system);
        scan_disks();

        memory.calculate();
        engine.calculate();
        growth.calculate();

        println!("\n===== RECURSIVE ENGINE =====");
        println!("Memory: {} | Active: {}", memory.value, memory.active);
        println!("Engine: {} | Active: {}", engine.value, engine.active);
        println!("Growth: {} | Active: {}", growth.value, growth.active);
        println!("============================");

        thread::sleep(Duration::from_secs(2));
    }
}
