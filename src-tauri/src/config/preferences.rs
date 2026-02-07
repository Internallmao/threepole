use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::ConfigFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ColorPreferences {
    pub completed_dot_color: String,
    pub incomplete_dot_color: String,
    pub notification_background_color: String,
    pub text_background_color: String,
    pub text_color: String,
    pub map_background_color: String,
}

impl Default for ColorPreferences {
    fn default() -> Self {
        Self {
            completed_dot_color: "#33ee33".to_string(),
            incomplete_dot_color: "#ee3333".to_string(),
            notification_background_color: "#12171c".to_string(),
            text_background_color: "rgba(0, 0, 0, 0.7)".to_string(),
            text_color: "#ffffff".to_string(),
            map_background_color: "#12171c".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct FilterPreferences {
    pub show_raids: bool,
    pub show_dungeons: bool,
    pub show_strikes: bool,
    pub show_lost_sectors: bool,
    pub show_completed: bool,
    pub show_incomplete: bool,
    pub show_fresh_start: bool,
    pub show_checkpoint: bool,
    pub min_duration_seconds: Option<u32>,
    pub max_duration_seconds: Option<u32>,
    pub specific_raids: HashMap<u32, bool>,
    pub specific_dungeons: HashMap<u32, bool>,
}

impl Default for FilterPreferences {
    fn default() -> Self {
        Self {
            show_raids: true,
            show_dungeons: true,
            show_strikes: true,
            show_lost_sectors: true,
            show_completed: true,
            show_incomplete: true,
            show_fresh_start: true,
            show_checkpoint: true,
            min_duration_seconds: None,
            max_duration_seconds: None,
            specific_raids: HashMap::new(),
            specific_dungeons: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SortPreferences {
    pub sort_by: String,
    pub sort_order: String,
    pub time_range: String,
}

impl Default for SortPreferences {
    fn default() -> Self {
        Self {
            sort_by: "time".to_string(),
            sort_order: "desc".to_string(),
            time_range: "all".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Preferences {
    pub enable_overlay: bool,
    pub display_daily_clears: bool,
    pub display_clear_notifications: bool,
    pub display_milliseconds: bool,
    pub colors: ColorPreferences,
    pub filters: FilterPreferences,
    pub sorting: SortPreferences,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enable_overlay: false,
            display_daily_clears: true,
            display_clear_notifications: true,
            display_milliseconds: false,
            colors: ColorPreferences::default(),
            filters: FilterPreferences::default(),
            sorting: SortPreferences::default(),
        }
    }
}

impl ConfigFile for Preferences {
    fn get_filename() -> &'static str {
        "preferences.json"
    }
}
