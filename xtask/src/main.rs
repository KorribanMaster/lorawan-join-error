//! Build orchestration tool for lora-experiment
//!
//! Provides custom Cargo commands for building and testing the embedded application.
//! Based on the xtask pattern from Ferrous Systems.

use std::env;
use std::path::PathBuf;
use xshell::cmd;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["build"] => build(),
        ["build", "--release"] => build_release(),
        ["flash"] => flash(),
        ["flash", "--release"] => flash_release(),
        ["test", "all"] => test_all(),
        ["test", "host"] => test_host(),
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn print_help() {
    println!("xtask - Build orchestration for lora-experiment");
    println!();
    println!("USAGE:");
    println!("    cargo xtask <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    build              Build the embedded application (debug)");
    println!("    build --release    Build the embedded application (release)");
    println!("    flash              Flash the embedded application to device (debug)");
    println!("    flash --release    Flash the embedded application to device (release)");
    println!("    test all           Run all tests (host + embedded build verification)");
    println!("    test host          Run host tests for victron-ble");
}

/// Build the embedded application in debug mode
fn build() -> Result<(), anyhow::Error> {
    println!("Building embedded application (debug)...");
    let _p = xshell::pushd(root_dir().join("cross").join("app"))?;
    cmd!("cargo +esp build").run()?;
    println!("Build complete!");
    Ok(())
}

/// Build the embedded application in release mode
fn build_release() -> Result<(), anyhow::Error> {
    println!("Building embedded application (release)...");
    let _p = xshell::pushd(root_dir().join("cross").join("app"))?;
    cmd!("cargo +esp build --release").run()?;
    println!("Build complete!");
    Ok(())
}

/// Flash the embedded application to device (debug)
fn flash() -> Result<(), anyhow::Error> {
    println!("Flashing embedded application (debug)...");
    let _p = xshell::pushd(root_dir().join("cross").join("app"))?;
    cmd!("cargo +esp run").run()?;
    Ok(())
}

/// Flash the embedded application to device (release)
fn flash_release() -> Result<(), anyhow::Error> {
    println!("Flashing embedded application (release)...");
    let _p = xshell::pushd(root_dir().join("cross").join("app"))?;
    cmd!("cargo +esp run --release").run()?;
    Ok(())
}

/// Run all tests: host tests and build verification
fn test_all() -> Result<(), anyhow::Error> {
    test_host()?;
    println!();
    println!("Running build verification...");
    build_release()?;
    println!();
    println!("All tests passed!");
    Ok(())
}

/// Run host tests for victron-ble
fn test_host() -> Result<(), anyhow::Error> {
    println!("Running host tests for victron-ble...");
    let _p = xshell::pushd(root_dir())?;
    cmd!("cargo test -p victron-ble").run()?;
    println!("All host tests passed!");
    Ok(())
}

/// Get the workspace root directory
fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
