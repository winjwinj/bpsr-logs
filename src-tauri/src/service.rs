use log::{info, warn};
use std::process::Command;

fn remove_windivert() -> bool {
    let output = Command::new("sc").args(["delete", "windivert"]).output();

    match output {
        Ok(output) if output.status.success() => {
            info!("deleted WinDivert driver service");
            true
        }
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout} {stderr}");
            // Error 1072 = service already marked for deletion, 1060 = service doesn't exist
            if !combined.contains("1072") && !combined.contains("1060") {
                let msg = combined.trim();
                if !msg.is_empty() {
                    warn!("could not delete WinDivert driver service: {msg}");
                }
            }
            false
        }
        Err(e) => {
            warn!("failed to execute delete command: {e}");
            false
        }
    }
}

pub fn stop_windivert() {
    info!("Cleaning up WinDivert resources...");
    let output = Command::new("sc").args(["stop", "windivert"]).output();

    let stopped = match output {
        Ok(output) if output.status.success() => {
            info!("stopped WinDivert driver service");
            true
        }
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout} {stderr}");
            // Error 1061 = service already stopping, 1060 = service doesn't exist
            if !combined.contains("1061") && !combined.contains("1060") {
                let msg = combined.trim();
                if !msg.is_empty() {
                    warn!("could not stop WinDivert driver service: {msg}");
                }
            }
            false
        }
        Err(e) => {
            warn!("failed to execute stop command: {e}");
            false
        }
    };

    if stopped {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    remove_windivert();
}
