//! Example: Running the IBM Logo ROM
//!
//! This example demonstrates how to load and run a simple CHIP-8 ROM
//! using our emulator components. It runs the IBM logo ROM which displays
//! the classic IBM logo sprite pattern.

use octo::{AsciiRenderer, Cpu, Display, Memory, Renderer};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CHIP-8 Emulator - IBM Logo Example");
    println!("==================================");

    // Load the IBM logo ROM
    let rom_path = "roms/ibm-logo.ch8";
    if !Path::new(rom_path).exists() {
        eprintln!("Error: ROM file '{}' not found", rom_path);
        eprintln!("Please make sure the IBM logo ROM is in the roms/ directory");
        return Ok(());
    }

    let rom_data = std::fs::read(rom_path)?;
    println!("Loaded ROM: {} ({} bytes)", rom_path, rom_data.len());

    // Initialize emulator components
    let mut memory = Memory::new(true); // Enable write protection
    let mut cpu = Cpu::new();
    let mut display = Display::new();

    // Load ROM into memory
    memory.load_rom(&rom_data)?;
    println!("ROM loaded at address 0x{:04X}", 0x200);

    // Run the program for a limited number of cycles
    println!("\nRunning emulator...");
    let mut cycles = 0;
    let max_cycles = 50; // Limit to prevent infinite loops

    loop {
        cycles += 1;
        if cycles > max_cycles {
            println!("Reached maximum cycles ({}), stopping", max_cycles);
            break;
        }

        // Execute one CPU cycle
        match cpu.execute_cycle(&mut memory, &mut display) {
            Ok(()) => {
                println!(
                    "Cycle {}: PC=0x{:04X}, I=0x{:04X}",
                    cycles,
                    cpu.get_pc(),
                    cpu.get_index()
                );

                // Check if we've hit the infinite loop at the end of IBM logo
                if cpu.get_pc() == 0x228 && cycles > 20 {
                    println!("Detected end-of-program loop, stopping");
                    break;
                }
            }
            Err(e) => {
                println!("Execution error at cycle {}: {}", cycles, e);
                break;
            }
        }

        // Add a small delay to make it easier to follow
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Display the final result
    println!("\nFinal Display Output:");
    println!("{}", "=".repeat(70));
    let renderer = AsciiRenderer;
    renderer.render(&display);
    println!("{}", "=".repeat(70));

    // Show some statistics
    let stats = display.get_stats();
    println!(
        "Display stats: {}/{} pixels on ({}%)",
        stats.pixels_on,
        stats.pixels_total,
        (stats.pixels_on * 100) / stats.pixels_total
    );

    println!("CPU final state:");
    println!("  PC: 0x{:04X}", cpu.get_pc());
    println!("  I:  0x{:04X}", cpu.get_index());
    println!("  V0: 0x{:02X}", cpu.get_register(0).unwrap_or(0));
    println!("  V1: 0x{:02X}", cpu.get_register(1).unwrap_or(0));

    println!("\nIBM Logo emulation complete!");
    Ok(())
}
