use std;

use fern;
use file_rotate;
use log::info;

use crate::lib_constant::EXIT_CODE_GENERAL;

fn map_log_level(user_level: &str) -> log::LevelFilter {
    let all_levels = [
        (log::LevelFilter::Off, "Off"),
        (log::LevelFilter::Error, "Error"),
        (log::LevelFilter::Warn, "Warn"),
        (log::LevelFilter::Info, "Info"),
        (log::LevelFilter::Debug, "Debug"),
        (log::LevelFilter::Trace, "Trace"),
    ];

    for (level, level_str) in all_levels {
        if user_level.to_lowercase() == level_str.to_lowercase() {
            return level;
        }
    }

    let all_levels_string: String = {
        let mut s = String::new();
        for (_, level_str) in all_levels {
            s.push_str(&level_str.to_lowercase());
            s.push_str(" ");
        }
        s.trim().into()
    };

    println!("Bad value [{}] for log level. Possible values are (caseless): {}", user_level, all_levels_string);
    std::process::exit(EXIT_CODE_GENERAL)
}

pub fn setup(log_base_path: &str, log_level: &str) {
    let log_level = map_log_level(log_level);

    let dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d] [%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level);

    if log_base_path == "-" {
        dispatch.chain(std::io::stdout()).apply().unwrap();
    } else {
        let log_base_path = std::path::PathBuf::from(&log_base_path);
        match log_base_path.parent() {
            None => (),
            Some(dir_path) => {
                println!("Creating log directory: {}", &dir_path.display());
                std::fs::create_dir_all(dir_path).expect("creating directory to be successful");
            }
        }
        let rotate_file = file_rotate::FileRotate::new(
            &log_base_path,
            file_rotate::suffix::TimestampSuffixScheme::default(file_rotate::suffix::FileLimit::MaxFiles(20)),
            file_rotate::ContentLimit::Bytes(1024 * 1024 * 10),
            file_rotate::compression::Compression::None,
        );
        dispatch
            .chain(Box::new(rotate_file) as Box<dyn std::io::Write + Send>)
            .apply()
            .unwrap();
    }

    info!("Log base path: {}", log_base_path);
    info!("Log level: {}", log_level);
}
