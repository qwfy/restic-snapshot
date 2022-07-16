use std::path::Path;
use std::process::Command;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use log::info;

use crate::lib_fs;
use crate::lib_cmd;

pub struct Snapshot {
    pub vol_path: String,
    pub mount_path: String,
}

pub fn create_snapshot(
    vol_group: &str, vol_name: &str, snapshot_name: &str,
    size: &str, // e.g. "100GiB"
    mount_parent: &str,
) -> Result<Snapshot> {
    let mount_dir = lib_fs::join!(mount_parent, snapshot_name);
    let mount_dir = lib_fs::path_to_string(mount_dir)?;

    let () = lib_fs::ensure_dir(&mount_dir)?;

    let src_path = get_lv_path(vol_group, vol_name)?;
    let mut lvcreate = Command::new("lvcreate");
    lvcreate
        .arg("--size")
        .arg(size)
        .arg("--snapshot")
        .arg("--name")
        .arg(snapshot_name)
        .arg(src_path);
    let _ = lib_cmd::run_success(&mut lvcreate)?;

    let snapshot_path = get_lv_path(vol_group, snapshot_name)?;

    let mut mount = Command::new("mount");
    mount
        .arg(&snapshot_path)
        .arg(&mount_dir);
    let _ = lib_cmd::run_success(&mut mount)?;

    Ok(Snapshot {
        vol_path: snapshot_path,
        mount_path: mount_dir,
    })
}


pub fn drop_snapshot(snapshot: Snapshot) -> Result<()> {
    let mut umount = Command::new("umount");
    umount
        .arg(&snapshot.mount_path);
    let _ = lib_cmd::run_success(&mut umount)?;

    lib_fs::remove_empty_dir(&snapshot.mount_path)?;

    let mut lvremove = Command::new("lvremove");
    lvremove
        .arg("--yes")
        .arg(snapshot.vol_path);
    lib_cmd::run_success(&mut lvremove)?;

    Ok(())
}

fn get_lv_path(vg: &str, lv: &str) -> Result<String> {
    let p = lib_fs::path_to_string(lib_fs::join!("/dev", vg, lv))?;
    Ok(p)
}