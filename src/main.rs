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
    let log_dir = "/mnt/large/restic/logs";
    let repo_dir = "/mnt/large/restic/repo";
    let cache_dir = "/mnt/large/restic/cache";
    let password_file = "/mnt/large/restic/password.txt";
    let restic_exe = "/mnt/large/restic/restic_0.13.0_linux_amd64";

    let vg = "godel";
    let lv = "home";

    let run_id = chrono::offset::Local::now().format("%Y_%m%d_%H%M%S").to_string();

    let log_file = PathBuf::from(log_dir);
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
        vg, lv, &run_id, "100GiB", "/mnt"
    )?;

    let mut restic = Command::new(restic_exe);
    restic
        .arg("--quiet")
        .arg("--repo")
        .arg(&repo_dir)
        .arg("--cache-dir")
        .arg(cache_dir)
        .arg("--password-file")
        .arg(&password_file)
        .arg("backup")
        .arg("--one-file-system")
        .arg("--tag")
        .arg("home")
        .arg(&snapshot.mount_path);
    lib_cmd::run_success(&mut restic)?;

    lib_lvm::drop_snapshot(snapshot)?;

    Ok(())
}