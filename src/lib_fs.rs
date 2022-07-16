use std::path::Path;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use log::info;

pub fn path_to_string<P: AsRef<Path>>(p: P) -> Result<String> {
    let p = p.as_ref();

    match p.to_str() {
        None => bail!("Cannot convert path [{}] to string", p.display()),
        Some(s) => Ok(String::from(s))
    }
}


pub fn ensure_dir<P: AsRef<Path>>(p: P) -> Result<()> {
    let p = p.as_ref();

    if p.is_dir() {
        Ok(())
    } else {
        if p.exists() {
            bail!("Path [{}] already exist", p.display())
        } else {
            info!("Creating directory: {}", p.display());
            std::fs::create_dir_all(p)
                .with_context(|| format!("Failed to ensure directory [{}]", p.display()))?;
            Ok(())
        }
    }
}

pub fn remove_empty_dir<P: AsRef<Path>>(p: P) -> Result<()> {
    let p = p.as_ref();
    info!("Removing directory [{}]", p.display());
    std::fs::remove_dir(p)?;
    info!("Removed directory [{}]", p.display());
    Ok(())
}

mod mac {
    macro_rules! join {
        ($base:expr, $($segment:expr),+) => {{
            let mut base: std::path::PathBuf = $base.into();
            $(
                base.push($segment);
            )+
            base
        }}
    }

    pub(crate) use join;
}

pub(crate) use mac::join;

