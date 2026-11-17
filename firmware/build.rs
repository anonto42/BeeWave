use std::process::Command;

/// First line of a command's stdout, or `fallback` when it cannot run (no git in a source tarball).
fn run(cmd: &str, args: &[&str], fallback: &str) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn main() {
    embuild::espidf::sysenv::output();
    // Build identity shown in Settings > OTA (`DeviceInfo`).
    println!(
        "cargo:rustc-env=BEEWAVE_GIT_SHA={}",
        run("git", &["rev-parse", "--short=7", "HEAD"], "unknown")
    );
    println!(
        "cargo:rustc-env=BEEWAVE_BUILT={}",
        run("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"], "unknown")
    );
    println!("cargo:rerun-if-changed=../.git/HEAD");
}
