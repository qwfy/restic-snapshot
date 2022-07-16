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
    std::process::exit(match run_app() {
        Ok(()) => 0,
        Err(e) => {
            error!("Backup failed: {}", e);
            lib_constant::EXIT_CODE_GENERAL
        }
    });
}


fn run_app() -> Result<()> {
    let config = lib_config::get();

    let run_id = chrono::offset::Local::now().format("%Y_%m%d_%H%M%S").to_string();

    let log_file = PathBuf::from(&config.app.log_dir);
    let log_file = log_file.join(format!("{}.log", run_id));

    info!("Starting run {}", &run_id);

    info!("Checking the current user");
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

    let snapshot = lib_lvm::create_snapshot(
        &config.sources[0].vg, &config.sources[0].lv,
        &run_id, &config.sources[0].snapshot_size, "/mnt"
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