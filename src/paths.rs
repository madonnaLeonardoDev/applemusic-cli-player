use std::path::PathBuf;
use std::sync::LazyLock;


pub static BROWSER_PROFILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
     std::env::var("HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("/tmp"))
    .join(".config/applemusic-daemon/browser-profile")
});

pub static SOCKET_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_daemon.sock")
});

pub static PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("applemusic_daemon.pid")
});