use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc, Datelike};
use serde::Serialize;
use tauri::{
    async_runtime::{self, JoinHandle},
    AppHandle, Manager,
};
use tokio::sync::Mutex;

use crate::{
    api::{
        requests::BungieResponseError,
        responses::{ActivityInfo, CompletedActivity, LatestCharacterActivity, ProfileInfo},
        Api, ApiError, Source,
    },
    config::profiles::Profile,
    consts::{DUNGEON_ACTIVITY_MODE, RAID_ACTIVITY_MODE, STRIKE_ACTIVITY_MODE, LOSTSECTOR_ACTIVITY_MODE},
    ConfigContainer, CacheContainer,
};

const KNOWN_RAID_HASHES: &[usize] = &[
    2122313384, 3213556450, 2693136600, 1042180643, 910380154,
    3881495763, 1441982566, 1374392663, 2381413764, 107319834,
    1541433876, 1044919065, 3817322389,
];

const KNOWN_DUNGEON_HASHES: &[usize] = &[
    2032534090, 2823159265, 2582501063, 4078656646, 1077850348,
    1262462921, 313828469, 300092127, 3834447244,
];

fn is_known_raid_hash(activity_hash: usize) -> bool {
    KNOWN_RAID_HASHES.contains(&activity_hash)
}

fn is_known_dungeon_hash(activity_hash: usize) -> bool {
    KNOWN_DUNGEON_HASHES.contains(&activity_hash)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    current_activity: CurrentActivity,
    activity_history: Vec<CompletedActivity>,
    profile_info: ProfileInfo,
}

#[derive(Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDataStatus {
    last_update: Option<PlayerData>,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CurrentActivity {
    start_date: DateTime<Utc>,
    activity_hash: usize,
    activity_info: Option<ActivityInfo>,
}

#[derive(Default)]
pub struct PlayerDataPoller {
    task_handle: Option<JoinHandle<()>>,
    current_playerdata: Arc<Mutex<PlayerDataStatus>>,
}

impl PlayerDataPoller {
    pub async fn reset(&mut self, app_handle: AppHandle) {
        if let Some(t) = self.task_handle.as_ref() {
            t.abort();
        }

        {
            let mut lock = self.current_playerdata.lock().await;
            *lock = PlayerDataStatus::default();

            send_data_update(&app_handle, lock.clone());
        }

        let playerdata_clone = self.current_playerdata.clone();

        self.task_handle = Some(async_runtime::spawn(async move {
            let profile = {
                let container = app_handle.state::<ConfigContainer>();
                let lock = container.0.lock().await;

                match &lock.get_profiles().selected_profile {
                    Some(p) => p.clone(),
                    None => {
                        let mut lock = playerdata_clone.lock().await;
                        lock.error = Some("No profile set".to_string());

                        send_data_update(&app_handle, lock.clone());
                        return;
                    }
                }
            };

            let profile_info = {
                let api = app_handle.state::<Api>();
                let mut lock = api.profile_info_source.lock().await;

                match lock.get(&profile).await {
                    Ok(p) => p,
                    Err(e) => {
                        let mut lock = playerdata_clone.lock().await;
                        lock.error = Some(format!("Failed to get profile info: {e}"));

                        send_data_update(&app_handle, lock.clone());
                        return;
                    }
                }
            };

            let mut current_activity = CurrentActivity {
                start_date: DateTime::<Utc>::MIN_UTC,
                activity_hash: 0,
                activity_info: None,
            };
            let mut activity_history = Vec::new();

            let res = match update_current(&app_handle, &mut current_activity, &profile).await {
                Ok(_) => update_history(&app_handle, &mut activity_history, &profile).await,
                Err(e) => Err(e),
            };

            {
                let mut lock = playerdata_clone.lock().await;
                match res {
                    Ok(_) => {
                        let playerdata = PlayerData {
                            current_activity: current_activity,
                            activity_history,
                            profile_info,
                        };

                        lock.last_update = Some(playerdata);
                        send_data_update(&app_handle, lock.clone());
                    }
                    Err(e) => {
                        lock.error = Some(e.to_string());
                        send_data_update(&app_handle, lock.clone());
                        return;
                    }
                }
            }

            let mut count = 0;

            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;

                let mut last_update = match playerdata_clone.lock().await.last_update.clone() {
                    Some(data) => data,
                    None => {
                        continue;
                    }
                };

                let res = if count < 5 {
                    update_current(&app_handle, &mut last_update.current_activity, &profile).await
                } else {
                    count = 0;
                    update_history(&app_handle, &mut last_update.activity_history, &profile).await
                };

                match res {
                    Ok(true) => {
                        let mut lock = playerdata_clone.lock().await;
                        lock.error = None;
                        lock.last_update = Some(last_update);

                        send_data_update(&app_handle, lock.clone())
                    }
                    Err(e) => {
                        let mut lock = playerdata_clone.lock().await;
                        lock.error = Some(e.to_string());

                        send_data_update(&app_handle, lock.clone())
                    }
                    _ => (),
                }

                count += 1;
            }
        }));
    }

    pub fn get_data(&mut self) -> Option<PlayerDataStatus> {
        return match &self.current_playerdata.try_lock() {
            Ok(p) => Some((*p).clone()),
            Err(_) => None,
        };
    }
}

fn send_data_update(handle: &AppHandle, data: PlayerDataStatus) {
    if let Some(o) = handle.get_window("overlay") {
        o.emit("playerdata_update", data.clone()).unwrap();
    }

    if let Some(o) = handle.get_window("details") {
        o.emit("playerdata_update", data).unwrap();
    }
}

async fn update_current(
    handle: &AppHandle,
    last_activity: &mut CurrentActivity,
    profile: &Profile,
) -> Result<bool> {
    let current_activities = Api::get_profile_activities(profile).await?;

    let activities = match current_activities.activities {
        Some(a) => a,
        None => bail!("Profile is private"),
    };

    let (characters, activities): (Vec<String>, Vec<LatestCharacterActivity>) =
        activities.into_iter().unzip();

    let latest_activity = activities
        .into_iter()
        .max()
        .ok_or(anyhow!("No character data for profile"))?;

    match last_activity
        .start_date
        .cmp(&latest_activity.date_activity_started)
    {
        std::cmp::Ordering::Less => {
            last_activity.start_date = latest_activity.date_activity_started
        }
        std::cmp::Ordering::Equal => {
            if last_activity.activity_info.is_none() {
                return Ok(false);
            }

            if last_activity.activity_hash == latest_activity.current_activity_hash {
                return Ok(false);
            }
        }
        std::cmp::Ordering::Greater => return Ok(false),
    }

    let api = handle.state::<Api>();

    api.profile_info_source
        .lock()
        .await
        .set_characters(profile, characters);

    if latest_activity.current_activity_hash == 0 {
        last_activity.activity_info = None;
        return Ok(true);
    }

    let current_activity_info = {
        let activity = api
            .activity_info_source
            .lock()
            .await
            .get(&latest_activity.current_activity_hash)
            .await;

        match activity {
            Ok(a) => a,
            Err(ApiError::ResponseError(BungieResponseError::ResponseMissing)) => {
                last_activity.activity_info = None;
                return Ok(true);
            }
            Err(e) => return Err(e.into()),
        }
    };

    if current_activity_info.name.is_empty() {
        last_activity.activity_info = None;
        return Ok(true);
    }

    last_activity.activity_hash = latest_activity.current_activity_hash;
    last_activity.activity_info = Some(current_activity_info);

    Ok(true)
}

async fn update_history(
    handle: &AppHandle,
    last_history: &mut Vec<CompletedActivity>,
    profile: &Profile,
) -> Result<bool> {
    let api = handle.state::<Api>();
    let cache_container = handle.state::<CacheContainer>();

    let profile_info = api.profile_info_source.lock().await.get(profile).await?;
    let profile_id = format!("{}_{}", profile.account_platform, profile.account_id);

    let now = chrono::Utc::now();
    let weekly_reset = get_destiny_weekly_reset_time(now);

    let mut cache_manager = cache_container.0.lock().await;
    
    let cached_activities = cache_manager.get_cached_activities(&profile_id);
    
    if let Some(cache) = cached_activities {
        #[cfg(debug_assertions)]
        println!("📦 Cache: Found {} cached activities for profile {}", cache.activities.len(), profile_id);
        
        let cache_age = now.signed_duration_since(cache.last_updated);
        let should_check_updates = cache_age.num_minutes() >= 5;
        
        if should_check_updates {
            #[cfg(debug_assertions)]
            println!("🔄 Cache: Checking for new activities (cache is {} minutes old)...", cache_age.num_minutes());
            let mut recent_activities: Vec<CompletedActivity> = Vec::new();
            
            for character_id in profile_info.character_ids.iter() {
                let history = Api::get_activity_history(profile, character_id, 0, 7).await?;
                if let Some(activities) = history.into_completed_activities() {
                    recent_activities.extend(activities);
                }
            }
            
            if cache_manager.has_new_activities(&profile_id, &recent_activities) {
                #[cfg(debug_assertions)]
                println!("🔄 Cache: New activities detected, fetching updates...");
                let mut new_activities: Vec<CompletedActivity> = Vec::new();
                
                for character_id in profile_info.character_ids.iter() {
                    for page in 0..5 {
                        let history = Api::get_activity_history(profile, character_id, page, 7).await?;
                        if let Some(activities) = history.into_completed_activities() {
                            if activities.is_empty() {
                                break;
                            }
                            new_activities.extend(activities);
                        } else {
                            break;
                        }
                    }
                }
                
                // Fetch PGCR data for new activities
                fetch_pgcrs_for_activities(&mut new_activities).await;
                
                cache_manager.merge_activities(profile_id.clone(), new_activities);
            } else {
                #[cfg(debug_assertions)]
                println!("✅ Cache: No new activities found");
            }
        } else {
            #[cfg(debug_assertions)]
            println!("✅ Cache: Using cached data (cache is {} minutes old, will check again in {} minutes)",
                cache_age.num_minutes(), 5 - cache_age.num_minutes());
        }
        
        let final_cache = cache_manager.get_cached_activities(&profile_id).unwrap();
        let mut all_activities = final_cache.activities.clone();
        
        all_activities.retain(|activity| {
            let is_raid_by_mode = activity.modes.iter().any(|m| *m == RAID_ACTIVITY_MODE);
            let is_dungeon_by_mode = activity.modes.iter().any(|m| *m == DUNGEON_ACTIVITY_MODE);
            let is_known_raid = is_known_raid_hash(activity.activity_hash);
            let is_known_dungeon = is_known_dungeon_hash(activity.activity_hash);
            
            let is_raid_or_dungeon = is_raid_by_mode || is_dungeon_by_mode || is_known_raid || is_known_dungeon;
            
            let is_strike_or_lost_sector = activity.modes.iter().any(|m| {
                *m == STRIKE_ACTIVITY_MODE || *m == LOSTSECTOR_ACTIVITY_MODE
            });

            if is_raid_or_dungeon {
                true
            } else if is_strike_or_lost_sector {
                activity.period >= weekly_reset
            } else {
                false
            }
        });
        
        if let Err(e) = cache_manager.save().await {
            #[cfg(debug_assertions)]
            eprintln!("Failed to save cache: {}", e);
        }
        
        if let Some(last) = last_history.iter().max() {
            if let Some(new) = all_activities.iter().max() {
                if last >= new {
                    return Ok(false);
                }
            }
        }

        all_activities.sort_by(|a, b| b.period.cmp(&a.period));
        *last_history = all_activities;
        
        return Ok(true);
    }
    
    #[cfg(debug_assertions)]
    println!("🔍 Cache: No cache found, performing full activity fetch...");
    #[cfg(debug_assertions)]
    println!("📊 Fetching activities for {} characters with concurrent requests", profile_info.character_ids.len());
    
    let mut all_activities = fetch_all_activities_concurrent(profile, &profile_info, weekly_reset, &mut cache_manager, &profile_id).await?;
    
    #[cfg(debug_assertions)]
    println!("🎉 Full fetch complete: {} total activities collected across all characters", all_activities.len());

    // Fetch PGCR data for all activities
    #[cfg(debug_assertions)]
    println!("💡 Note: You can use the app while PGCR data is being fetched in the background");
    #[cfg(debug_assertions)]
    println!("💡 Duration filters work immediately, checkpoint filters will work once PGCR fetch completes");
    fetch_pgcrs_for_activities(&mut all_activities).await;

    #[cfg(debug_assertions)]
    println!("💾 Cache: Saving final cache with {} activities...", all_activities.len());
    cache_manager.update_cache(profile_id.clone(), all_activities.clone());
    if let Err(e) = cache_manager.save().await {
        #[cfg(debug_assertions)]
        eprintln!("❌ Cache: Failed to save final cache: {}", e);
    } else {
        #[cfg(debug_assertions)]
        println!("✅ Cache: Final cache saved successfully!");
    }

    if let Some(last) = last_history.iter().max() {
        if let Some(new) = all_activities.iter().max() {
            if last >= new {
                return Ok(false);
            }
        }
    }

    all_activities.sort_by(|a, b| b.period.cmp(&a.period));

    *last_history = all_activities;

    Ok(true)
}

async fn fetch_pgcrs_for_activities(activities: &mut Vec<CompletedActivity>) {
    use tokio::sync::Semaphore;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;
    
    let total_activities = activities.len();
    
    // Count activities that need PGCR fetch (only those without PGCR data)
    let needs_fetch = activities.iter()
        .filter(|a| a.activity_was_started_from_beginning.is_none())
        .count();
    
    if needs_fetch == 0 {
        #[cfg(debug_assertions)]
        println!("✅ PGCR: All {} activities already have PGCR data, skipping fetch", total_activities);
        return;
    }
    
    #[cfg(debug_assertions)]
    println!("🎮 PGCR: Fetching PGCR data for {} activities (skipping {} already cached)...",
        needs_fetch, total_activities - needs_fetch);
    #[cfg(debug_assertions)]
    println!("⏱️  PGCR: Using 75 concurrent requests for maximum throughput");
    #[cfg(debug_assertions)]
    println!("📊 PGCR: Progress updates every 50 activities...");
    
    let start_time = std::time::Instant::now();
    let fetched = Arc::new(TokioMutex::new(0usize));
    let failed = Arc::new(TokioMutex::new(0usize));
    
    // Use 75 concurrent requests for faster fetching
    let semaphore = Arc::new(Semaphore::new(75));
    
    // Collect ONLY activities that need PGCR fetch (missing activityWasStartedFromBeginning)
    let fetch_list: Vec<(usize, String)> = activities.iter()
        .enumerate()
        .filter(|(_, a)| a.activity_was_started_from_beginning.is_none())
        .map(|(i, a)| (i, a.instance_id.clone()))
        .collect();
    
    let total_to_fetch = fetch_list.len();
    let mut handles = vec![];
    
    for (fetch_index, (activity_index, instance_id)) in fetch_list.into_iter().enumerate() {
        let semaphore = semaphore.clone();
        let fetched = fetched.clone();
        let failed = failed.clone();
        let start_time_clone = start_time.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            // Progress update every 50 activities
            #[cfg(debug_assertions)]
            if fetch_index > 0 && fetch_index % 50 == 0 {
                let elapsed = start_time_clone.elapsed().as_secs();
                let rate = if elapsed > 0 { fetch_index as f64 / elapsed as f64 } else { 0.0 };
                let remaining = total_to_fetch - fetch_index;
                let eta = if rate > 0.0 { (remaining as f64 / rate) as u64 } else { 0 };
                let f = *fetched.lock().await;
                let fail = *failed.lock().await;
                println!("   📊 Progress: {}/{} ({:.1}%) - Rate: {:.1}/s - ETA: {}s - Success: {}, Failed: {}",
                    fetch_index, total_to_fetch, (fetch_index as f64 / total_to_fetch as f64) * 100.0,
                    rate, eta, f, fail);
            }
            
            match Api::get_pgcr(&instance_id).await {
                Ok(pgcr) => {
                    *fetched.lock().await += 1;
                    Some((activity_index, pgcr))
                }
                Err(e) => {
                    let fail_count = {
                        let mut f = failed.lock().await;
                        *f += 1;
                        *f
                    };
                    #[cfg(debug_assertions)]
                    if fail_count <= 10 {
                        eprintln!("   ⚠️ Failed to fetch PGCR for activity {}: {}", instance_id, e);
                    } else if fail_count == 11 {
                        eprintln!("   ⚠️ Suppressing further error messages...");
                    }
                    None
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete and update activities
    #[cfg(debug_assertions)]
    println!("⏳ PGCR: Waiting for {} concurrent requests to complete...", handles.len());
    for handle in handles {
        if let Ok(Some((activity_index, pgcr))) = handle.await {
            if let Some(activity) = activities.get_mut(activity_index) {
                activity.starting_phase_index = pgcr.starting_phase_index;
                activity.activity_was_started_from_beginning = pgcr.activity_was_started_from_beginning;
            }
        }
    }
    
    #[cfg(debug_assertions)]
    {
        let elapsed = start_time.elapsed();
        let f = *fetched.lock().await;
        let fail = *failed.lock().await;
        let rate = if elapsed.as_secs() > 0 { f as f64 / elapsed.as_secs_f64() } else { 0.0 };
        println!("✅ PGCR: Completed in {:.1}s - Success: {}, Failed: {}, Rate: {:.1}/s",
            elapsed.as_secs_f64(), f, fail, rate);
    }
}

async fn fetch_all_activities_concurrent(
    profile: &Profile,
    profile_info: &ProfileInfo,
    weekly_reset: DateTime<Utc>,
    cache_manager: &mut tokio::sync::MutexGuard<'_, crate::cache::CacheManager>,
    profile_id: &str,
) -> Result<Vec<CompletedActivity>> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;
    
    let all_activities = Arc::new(TokioMutex::new(Vec::new()));
    
    // Use 30 concurrent requests for activity history fetching
    let semaphore = Arc::new(Semaphore::new(30));
    let mut handles = vec![];
    
    #[cfg(debug_assertions)]
    println!("📊 Starting concurrent fetch with 30 parallel requests across {} characters", profile_info.character_ids.len());
    
    for (char_index, character_id) in profile_info.character_ids.iter().enumerate() {
        let character_id = character_id.clone();
        let profile = profile.clone();
        let all_activities = all_activities.clone();
        let semaphore = semaphore.clone();
        let char_count = profile_info.character_ids.len();
        let weekly_reset = weekly_reset.clone();
        
        let handle = tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("👤 Character {}/{}: Starting fetch for character ID {}", char_index + 1, char_count, character_id);
            
            // Spawn 10 concurrent workers per character
            let mut worker_handles = vec![];
            let next_page = Arc::new(TokioMutex::new(0usize));
            let should_stop = Arc::new(TokioMutex::new(false));
            let total_collected = Arc::new(TokioMutex::new(0usize));
            
            for _worker_id in 0..10 {
                let semaphore = semaphore.clone();
                let profile = profile.clone();
                let character_id = character_id.clone();
                let all_activities = all_activities.clone();
                let weekly_reset = weekly_reset.clone();
                let next_page = next_page.clone();
                let should_stop = should_stop.clone();
                let total_collected = total_collected.clone();
                
                let worker_handle = tokio::spawn(async move {
                    loop {
                        // Check if we should stop
                        if *should_stop.lock().await {
                            break;
                        }
                        
                        // Get next page to fetch
                        let page = {
                            let mut np = next_page.lock().await;
                            if *np >= 1250 {
                                break;
                            }
                            let p = *np;
                            *np += 1;
                            p
                        };
                        
                        let _permit = semaphore.acquire().await.unwrap();
                        
                        let history = match Api::get_activity_history(&profile, &character_id, page, 7).await {
                            Ok(h) => h,
                            Err(_) => {
                                *should_stop.lock().await = true;
                                break;
                            }
                        };
                        
                        drop(_permit); // Release permit immediately after API call
                        
                        let activities = match history.into_completed_activities() {
                            Some(a) => a,
                            None => {
                                *should_stop.lock().await = true;
                                break;
                            }
                        };
                        
                        if activities.is_empty() {
                            *should_stop.lock().await = true;
                            break;
                        }
                        
                        let mut collected = 0;
                        
                        for activity in activities.into_iter() {
                            let is_raid_by_mode = activity.modes.iter().any(|m| *m == RAID_ACTIVITY_MODE);
                            let is_dungeon_by_mode = activity.modes.iter().any(|m| *m == DUNGEON_ACTIVITY_MODE);
                            let is_known_raid = is_known_raid_hash(activity.activity_hash);
                            let is_known_dungeon = is_known_dungeon_hash(activity.activity_hash);
                            
                            let is_raid_or_dungeon = is_raid_by_mode || is_dungeon_by_mode || is_known_raid || is_known_dungeon;
                            
                            let is_strike = activity.modes.iter().any(|m| *m == STRIKE_ACTIVITY_MODE);
                            let is_lost_sector = activity.modes.iter().any(|m| *m == LOSTSECTOR_ACTIVITY_MODE);
                            let is_strike_or_lost_sector = is_strike || is_lost_sector;
                            
                            if is_raid_or_dungeon {
                                all_activities.lock().await.push(activity);
                                collected += 1;
                            } else if is_strike_or_lost_sector && activity.period >= weekly_reset {
                                all_activities.lock().await.push(activity);
                                collected += 1;
                            }
                        }
                        
                        *total_collected.lock().await += collected;
                    }
                });
                
                worker_handles.push(worker_handle);
            }
            
            // Wait for all workers to complete
            for handle in worker_handles {
                let _ = handle.await;
            }
            
            let final_page = *next_page.lock().await;
            let final_collected = *total_collected.lock().await;
            
            #[cfg(debug_assertions)]
            println!("   ✅ Character {}/{}: Completed {} pages - {} activities collected",
                char_index + 1, char_count, final_page, final_collected);
        });
        
        handles.push(handle);
    }
    
    // Wait for all character fetches to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let mut all_activities = Arc::try_unwrap(all_activities).unwrap().into_inner();
    
    #[cfg(debug_assertions)]
    println!("🎉 Concurrent fetch complete: {} total activities collected", all_activities.len());
    
    // Fetch PGCR data for all activities
    #[cfg(debug_assertions)]
    println!("💡 Note: You can use the app while PGCR data is being fetched in the background");
    #[cfg(debug_assertions)]
    println!("💡 Duration filters work immediately, checkpoint filters will work once PGCR fetch completes");
    fetch_pgcrs_for_activities(&mut all_activities).await;
    
    #[cfg(debug_assertions)]
    println!("💾 Cache: Saving final cache with {} activities...", all_activities.len());
    cache_manager.update_cache(profile_id.to_string(), all_activities.clone());
    if let Err(e) = cache_manager.save().await {
        #[cfg(debug_assertions)]
        eprintln!("❌ Cache: Failed to save final cache: {}", e);
    } else {
        #[cfg(debug_assertions)]
        println!("✅ Cache: Final cache saved successfully!");
    }
    
    Ok(all_activities)
}

fn get_destiny_weekly_reset_time(date: DateTime<Utc>) -> DateTime<Utc> {
    let mut reset_time = DateTime::<Utc>::from_utc(
        date.date_naive().and_hms_opt(17, 0, 0).unwrap(),
        Utc
    );
    
    if date < reset_time {
        reset_time = reset_time - chrono::Duration::days(1);
    }
    
    let days_since_tuesday = (reset_time.weekday().num_days_from_monday() + 5) % 7;
    reset_time - chrono::Duration::days(days_since_tuesday as i64)
}
