use std::path::PathBuf;
use std::sync::LazyLock;

pub const NAME:&str = "applemusic-cli";

pub static BROWSER_PROFILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
     std::env::var("HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("~/"))
    .join(".config/applemusic-daemon/browser-profile")
});

pub static DAEMON_SOCKET_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_daemon.sock")
});

pub static DAEMON_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_daemon.pid")
});

pub static GECKO_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_gecko.pid")
});

pub static BROWSER_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_browser.pid")
});

