use std::io;
use std::process::Command;
use std::process::Output;

use anyhow::bail;
use anyhow::Result;
use anyhow::Error;
use thiserror;
use log::info;
use log::warn;
use regex::Regex;


#[derive(thiserror::Error, Debug)]
enum RunCommandError {

    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error("Command exited with unsuccessful exit code: {}", .0.status.code().map(|x| x.to_string()).unwrap_or(String::from("<Exit Code Unavailable>")))]
    ExitCodeIsUnsuccessful(Output),
}


/// Run the command, expect it to be success.
pub fn run_success(cmd: &mut Command) -> Result<Output> {
    info!("Running command: {:?}", &cmd);
    let output = cmd.output();

    match output {
        Err(e) => {
            warn!("Failed to run command, error is: {}", &e);
            Err(Error::new(RunCommandError::IOError(e)))
        },

        Ok(output) => {
            let exit_code = output.status.code()
                .map(|i| i.to_string())
                .unwrap_or(String::from("<Exit Code Unavailable>"));
            let stdout = drop_enclosing_new_lines(best_effort_decode_to_string(output.stdout.clone()));
            let stderr = drop_enclosing_new_lines(best_effort_decode_to_string(output.stderr.clone()));

            info!("Command exit code: {}", &exit_code);
            info!("Command stdout: ==================================\n{}\n==================================================", stdout);
            info!("Command stderr: ==================================\n{}\n==================================================", stderr);

            if output.status.success() {
                info!("Command finished successfully");
                Ok(output)
            } else {
                warn!("Command failed with status code: {}", &exit_code);
                Err(Error::new(RunCommandError::ExitCodeIsUnsuccessful(output)))
            }
        }
    }
}


fn best_effort_decode_to_string(bytes: Vec<u8>) -> String {
    let bytes_copy = bytes.clone();

    match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => {
            let mut lossy = String::from_utf8_lossy(&bytes_copy).to_string();
            lossy.insert_str(0, "<Best Effort Decoding> ");
            lossy
        }
    }
}

fn drop_enclosing_new_lines(s: String) -> String {
    let p = Regex::new(r"^[\r\n]+").unwrap();
    let s = p.replace_all(&s, "").to_string();

    let p = Regex::new(r"[\r\n]+$").unwrap();
    let s = p.replace_all(&s, "").to_string();

    s
}