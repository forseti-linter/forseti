use anyhow::Result;
use std::path::PathBuf;
use std::{thread, time};

pub fn run(cache_path: &PathBuf, enable_cache: bool, path: &PathBuf, force: bool) -> Result<()> {
    // Annoying
    _ = cache_path;
    _ = enable_cache;
    _ = path;
    _ = force;

    let fake_time = time::Duration::from_secs(3);

    println!("Discovering engines...");
    thread::sleep(fake_time);
    println!("Discovering rulesets...");
    thread::sleep(fake_time);

    println!("Installing engines...");
    thread::sleep(fake_time);
    println!("Installing rulesets...");
    thread::sleep(fake_time);

    println!("Everything install successfully!");
    Ok(())
}
