use std::{collections::HashMap, path::PathBuf};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::api::responses::CompletedActivity;

const CACHE_VERSION: u32 = 2; // Increment this to invalidate old caches

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivityCache {
    pub activities: Vec<CompletedActivity>,
    pub last_updated: DateTime<Utc>,
    pub profile_id: String,
    #[serde(default)]
    pub cache_version: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CacheManager {
    pub profiles: HashMap<String, ActivityCache>,
    #[serde(default)]
    pub version: u32,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            version: CACHE_VERSION,
        }
    }

    pub async fn load() -> Result<Self> {
        let cache_path = Self::get_cache_path()?;
        
        if !cache_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&cache_path).await?;
        
        match serde_json::from_str::<CacheManager>(&content) {
            Ok(cache) => {
                // Check cache version
                if cache.version != CACHE_VERSION {
                    #[cfg(debug_assertions)]
                    println!("🗑️ Cache: Invalidating old cache (version {} -> {})", cache.version, CACHE_VERSION);
                    if let Err(delete_err) = fs::remove_file(&cache_path).await {
                        #[cfg(debug_assertions)]
                        println!("⚠️ Cache: Failed to delete old cache file: {}", delete_err);
                    }
                    return Ok(Self::new());
                }
                
                // Check individual profile cache versions
                let mut valid_cache = cache;
                valid_cache.profiles.retain(|profile_id, activity_cache| {
                    if activity_cache.cache_version != CACHE_VERSION {
                        #[cfg(debug_assertions)]
                        println!("🗑️ Cache: Removing outdated cache for profile {} (version {} -> {})",
                            profile_id, activity_cache.cache_version, CACHE_VERSION);
                        false
                    } else {
                        true
                    }
                });
                
                Ok(valid_cache)
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("🗑️ Cache: Removing incompatible cache file due to schema change: {}", e);
                if let Err(delete_err) = fs::remove_file(&cache_path).await {
                    #[cfg(debug_assertions)]
                    println!("⚠️ Cache: Failed to delete old cache file: {}", delete_err);
                }
                Ok(Self::new())
            }
        }
    }

    pub async fn save(&self) -> Result<()> {
        let cache_path = Self::get_cache_path()?;
        
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&cache_path, content).await?;
        
        #[cfg(debug_assertions)]
        {
            let profile_count = self.profiles.len();
            let total_activities: usize = self.profiles.values().map(|c| c.activities.len()).sum();
            println!("💾 Cache: Saved cache to {:?} with {} profiles and {} total activities", cache_path, profile_count, total_activities);
        }
        
        Ok(())
    }

    pub fn get_cached_activities(&self, profile_id: &str) -> Option<&ActivityCache> {
        self.profiles.get(profile_id)
    }

    pub fn update_cache(&mut self, profile_id: String, activities: Vec<CompletedActivity>) {
        let cache = ActivityCache {
            activities,
            last_updated: Utc::now(),
            profile_id: profile_id.clone(),
            cache_version: CACHE_VERSION,
        };
        
        self.version = CACHE_VERSION;
        self.profiles.insert(profile_id, cache);
    }

    pub fn merge_activities(&mut self, profile_id: String, new_activities: Vec<CompletedActivity>) {
        if let Some(existing_cache) = self.profiles.get_mut(&profile_id) {
            let mut all_activities = existing_cache.activities.clone();
            
            for new_activity in new_activities {
                if !all_activities.iter().any(|existing| {
                    existing.instance_id == new_activity.instance_id && 
                    existing.period == new_activity.period
                }) {
                    all_activities.push(new_activity);
                }
            }
            
            all_activities.sort_by(|a, b| b.period.cmp(&a.period));
            
            existing_cache.activities = all_activities;
            existing_cache.last_updated = Utc::now();
            existing_cache.cache_version = CACHE_VERSION;
            self.version = CACHE_VERSION;
        } else {
            self.update_cache(profile_id, new_activities);
        }
    }

    #[allow(dead_code)]
    pub fn should_refresh_cache(&self, profile_id: &str, max_age_hours: i64) -> bool {
        if let Some(cache) = self.profiles.get(profile_id) {
            let age = Utc::now().signed_duration_since(cache.last_updated);
            age.num_hours() >= max_age_hours
        } else {
            true
        }
    }

    pub fn has_new_activities(&self, profile_id: &str, recent_activities: &[CompletedActivity]) -> bool {
        if let Some(cache) = self.profiles.get(profile_id) {
            if cache.activities.is_empty() || recent_activities.is_empty() {
                return !recent_activities.is_empty();
            }
            
            let most_recent_cached = &cache.activities[0];
            for activity in recent_activities {
                if activity.period > most_recent_cached.period {
                    return true;
                }
                if activity.period == most_recent_cached.period &&
                   activity.instance_id != most_recent_cached.instance_id {
                    return true;
                }
            }
            
            false
        } else {
            !recent_activities.is_empty()
        }
    }

    #[allow(dead_code)]
    pub fn get_most_recent_activity_time(&self, profile_id: &str) -> Option<String> {
        self.profiles.get(profile_id)
            .and_then(|cache| cache.activities.first())
            .map(|activity| activity.period.to_rfc3339())
    }

    #[allow(dead_code)]
    pub fn get_cache_cutoff_date(&self, profile_id: &str) -> Option<DateTime<Utc>> {
        self.profiles.get(profile_id).map(|cache| cache.last_updated)
    }

    fn get_cache_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        path.push("threepole");
        path.push("activity_cache.json");
        
        Ok(path)
    }

    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.profiles.clear();
    }

    #[allow(dead_code)]
    pub async fn clear_cache_directory() -> Result<()> {
        let cache_path = Self::get_cache_path()?;
        
        if cache_path.exists() {
            fs::remove_file(&cache_path).await?;
            #[cfg(debug_assertions)]
            println!("🗑️ Cache: Removed cache file at {:?}", cache_path);
        }
        
        if let Some(parent) = cache_path.parent() {
            if parent.exists() {
                if let Ok(mut entries) = fs::read_dir(parent).await {
                    let mut has_files = false;
                    while let Ok(Some(_)) = entries.next_entry().await {
                        has_files = true;
                        break;
                    }
                    
                    if !has_files {
                        if let Err(e) = fs::remove_dir(parent).await {
                            #[cfg(debug_assertions)]
                            println!("⚠️ Cache: Could not remove empty cache directory: {}", e);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("🗑️ Cache: Removed empty cache directory at {:?}", parent);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_profile_cache(&mut self, profile_id: &str) {
        self.profiles.remove(profile_id);
    }

    #[allow(dead_code)]
    pub fn get_cache_stats(&self) -> HashMap<String, (usize, DateTime<Utc>)> {
        self.profiles.iter().map(|(id, cache)| {
            (id.clone(), (cache.activities.len(), cache.last_updated))
        }).collect()
    }
}