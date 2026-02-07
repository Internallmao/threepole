use std::time::Duration;

pub const TARGET_NAME: &str = "destiny2.exe";
pub const OVERLAY_POLL_INTERVAL: Duration = Duration::from_millis(500);
pub const APP_NAME: &str = "threepole";
pub const APP_VER: &str = env!("CARGO_PKG_VERSION");
pub fn get_api_key() -> String {
    std::env::var("BUNGIE_API_KEY").unwrap_or_else(|_| "5f193ccb77dd424583b1c19413424e43".to_string())
}
pub const API_PATH: &str = "https://www.bungie.net/Platform";
pub const NAMED_PIPE: &str = r"\\.\pipe\threepole-open";
pub const USER_AGENT: &str = concat!("threepole/", env!("CARGO_PKG_VERSION"));

pub const RAID_ACTIVITY_MODE: usize = 4;
pub const DUNGEON_ACTIVITY_MODE: usize = 82;
pub const STRIKE_ACTIVITY_MODE: usize = 18;
pub const LOSTSECTOR_ACTIVITY_MODE: usize = 87;

pub const RAID_ACTIVITY_HASH: usize = 2043403989;

// Polling intervals
pub const POLLER_INTERVAL_SECS: u64 = 5;
pub const POLLER_HISTORY_CHECK_INTERVAL: usize = 5;
pub const CACHE_STALE_MINUTES: i64 = 5;

// API pagination
pub const ACTIVITY_HISTORY_PAGE_SIZE: usize = 7;

// Concurrency limits
pub const ACTIVITY_FETCH_CONCURRENCY: usize = 30;
pub const ACTIVITY_FETCH_WORKERS: usize = 10;
pub const ACTIVITY_FETCH_MAX_PAGES: usize = 1250;
pub const PGCR_FETCH_CONCURRENCY: usize = 75;
#[cfg(debug_assertions)]
pub const PGCR_PROGRESS_INTERVAL: usize = 50;
#[cfg(debug_assertions)]
pub const PGCR_ERROR_LOG_LIMIT: usize = 10;

// Destiny time constants
pub const DESTINY_DAILY_RESET_HOUR: u32 = 17;
