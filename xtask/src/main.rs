//! Build orchestration tool for lora-experiment
//!
//! Provides custom Cargo commands for building and testing the embedded application.

use std::process::Command;

fn main() {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some("build") => build_embedded(),
        Some("test") => run_host_tests(),
        _ => print_help(),
    }
}

fn build_embedded() {
    println!("Building embedded application in cross/app...");
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir("cross/app")
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        std::process::exit(1);
    }
    println!("Embedded build complete!");
}

fn run_host_tests() {
    println!("Running host tests for victron-protocol...");
    let status = Command::new("cargo")
        .args(&["test", "--package", "victron-protocol"])
        .status()
        .expect("Failed to execute cargo test");

    if !status.success() {
        std::process::exit(1);
    }
    println!("All tests passed!");
}

fn print_help() {
    println!("xtask - Build orchestration for lora-experiment");
    println!();
    println!("USAGE:");
    println!("    cargo xtask <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    build    Build the embedded application");
    println!("    test     Run host tests for victron-protocol");
}
