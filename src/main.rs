use std::fmt::format;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use chrono;
use log::info;
use log::error;
use anyhow::Result;
use anyhow::bail;
use anyhow::Error;

mod lib_cmd;
mod lib_fs;
mod lib_log;
mod lib_constant;
mod lib_lvm;
mod lib_config;


fn main() {
    std::process::exit(match run_sources() {
        Ok(()) => 0,
        Err(e) => {
            error!("Backup failed: {}", e);
            lib_constant::EXIT_CODE_GENERAL
        }
    });
}

fn run_sources() -> Result<()> {
    let run_id = chrono::offset::Local::now().format("%Y_%m%d_%H%M%S").to_string();

    // Get the default config file path
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let config_file = exe_dir.join("restic-snapshot.toml");
    let config_file = lib_fs::path_to_string(&config_file)?;
    lib_config::setup(&config_file);
    let config = lib_config::get();

    // Setup logging
    let log_file = PathBuf::from(&config.app.log_dir);
    let log_file = log_file.join(format!("{}.log", run_id));
    lib_log::setup(&lib_fs::path_to_string(&log_file)?, "debug");

    info!("Using run id {}", &run_id);

    info!("Ensure the current user is root");
    let mut current_user = Command::new("id");
    current_user.arg("-u");
    let current_user_out = lib_cmd::run_success(&mut current_user)?;
    let current_user_id: i32 = String::from_utf8(current_user_out.stdout)
        .expect("Decoding the output of id command to string")
        .parse()
        .expect("Parse the output of id command to integer");
    if current_user_id != 0 {
        bail!("To be able to backup files properly, this command need to be run as root");
    }

    for i in 0..config.sources.len() {
        let one_id = format!("{}-{:02}", &run_id, i);

        // TODO: Continue on error?
        run_one(i, one_id)?;
    }

    Ok(())
}


fn run_one(i: usize, run_id: String) -> Result<()> {
    let config = lib_config::get();

    let snapshot = lib_lvm::create_snapshot(
        &config.sources[i].vg, &config.sources[i].lv,
        &run_id, &config.sources[i].snapshot_size, "/mnt"
    )?;

    let mut restic = Command::new(&config.restic.exe_path);
    restic
        .arg("--quiet")
        .arg("--repo")
        .arg(&config.restic.repo_dir)
        .arg("--cache-dir")
        .arg(&config.restic.cache_dir)
        .arg("--password-file")
        .arg(&config.restic.password_file)
        .arg("backup")
        .arg("--one-file-system")
        .arg("--tag")
        .arg("home")
        .arg(&snapshot.mount_path);
    lib_cmd::run_success(&mut restic)?;

    lib_lvm::drop_snapshot(snapshot)?;

    Ok(())
}