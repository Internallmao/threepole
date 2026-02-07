#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::io;

use api::{
    responses::{ActivityInfo, BungieProfile, ProfileInfo},
    Api, Source,
};
use cache::CacheManager;
use config::{
    preferences::Preferences,
    profiles::{Profile, Profiles},
    ConfigManager,
};
use consts::{APP_NAME, APP_VER, NAMED_PIPE};
use pollers::{
    overlay::overlay_poller,
    playerdata::{PlayerDataPoller, PlayerDataStatus},
};
use tauri::{
    async_runtime::{self, JoinHandle},
    AppHandle, CustomMenuItem, Manager, RunEvent, State, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl,
};
use tokio::{
    net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions},
    sync::Mutex,
};

mod api;
mod cache;
mod config;
mod consts;
mod pollers;

struct ConfigContainer(Mutex<ConfigManager>);

struct CacheContainer(Mutex<CacheManager>);

#[derive(Default)]
struct PlayerDataPollerContainer(Mutex<PlayerDataPoller>);

#[derive(Default)]
struct OverlayPollerHandle(Mutex<Option<JoinHandle<()>>>);

#[tauri::command]
async fn open_preferences(handle: AppHandle) -> Result<(), tauri::Error> {
    open_preferences_window(&handle)
}

#[tauri::command]
async fn open_profiles(handle: AppHandle) -> Result<(), tauri::Error> {
    open_profiles_window(&handle)
}

#[tauri::command]
async fn get_preferences(container: State<'_, ConfigContainer>) -> Result<Preferences, ()> {
    Ok(container.0.lock().await.get_preferences().clone())
}

#[tauri::command]
async fn set_preferences(
    handle: AppHandle,
    preferences: Preferences,
    container: State<'_, ConfigContainer>,
    poller_handle: State<'_, OverlayPollerHandle>,
) -> Result<(), String> {
    let mut lock = container.0.lock().await;
    lock.set_preferences(preferences.clone())
        .map_err(|e| e.to_string())?;

    if let Some(o) = handle.get_window("overlay") {
        if preferences.enable_overlay {
            let _ = o.emit("preferences_update", preferences);
        } else {
            if let Some(h) = poller_handle.0.lock().await.as_ref() {
                h.abort();
            }

            let _ = o.close();
        }
    } else if preferences.enable_overlay {
        let _ = create_overlay(handle).await;
    }

    Ok(())
}

#[tauri::command]
async fn get_profiles(container: State<'_, ConfigContainer>) -> Result<Profiles, ()> {
    Ok(container.0.lock().await.get_profiles().clone())
}

#[tauri::command]
async fn set_profiles(
    handle: AppHandle,
    profiles: Profiles,
    config_container: State<'_, ConfigContainer>,
    poller_container: State<'_, PlayerDataPollerContainer>,
) -> Result<(), String> {
    let mut lock = config_container.0.lock().await;

    let was_no_profile = lock.get_profiles().selected_profile.is_none();

    lock.set_profiles(profiles).map_err(|e| e.to_string())?;

    if was_no_profile {
        if handle.get_window("overlay").is_none() && lock.get_preferences().enable_overlay {
            let _ = create_overlay(handle.clone()).await;
        }

        let _ = open_details_window(&handle, true);
    }

    poller_container.0.lock().await.reset(handle).await;

    Ok(())
}

#[tauri::command]
async fn get_profile_info(profile: Profile, api: State<'_, Api>) -> Result<ProfileInfo, String> {
    Ok(api
        .profile_info_source
        .lock()
        .await
        .get(&profile)
        .await
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_activity_info(
    activity_hash: usize,
    api: State<'_, Api>,
) -> Result<ActivityInfo, String> {
    Ok(api
        .activity_info_source
        .lock()
        .await
        .get(&activity_hash)
        .await
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn search_profile(
    display_name: String,
    display_name_code: usize,
) -> Result<Vec<BungieProfile>, String> {
    Ok(Api::search_profile(&display_name, display_name_code)
        .await
        .map_err(|e| e.to_string())?)
}

async fn create_overlay(handle: AppHandle) -> Result<(), tauri::Error> {
    let overlay = WindowBuilder::new(
        &handle,
        "overlay",
        WindowUrl::App("./src/overlay/overlay.html".into()),
    )
    .title(APP_NAME)
    .transparent(true)
    .decorations(false)
    .fullscreen(true)
    .resizable(false)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .build()?;

    overlay.set_ignore_cursor_events(true)?;

    #[cfg(debug_assertions)]
    overlay.open_devtools();

    let handle_clone = handle.clone();
    let poller_handle = handle.state::<OverlayPollerHandle>();
    let mut lock = poller_handle.0.lock().await;

    if let Some(h) = lock.as_ref() {
        h.abort();
    }

    let handle = async_runtime::spawn(async move { overlay_poller(handle_clone).await });

    *lock = Some(handle);

    Ok(())
}

#[tauri::command]
async fn get_playerdata(
    poller_container: State<'_, PlayerDataPollerContainer>,
) -> Result<Option<PlayerDataStatus>, ()> {
    Ok(poller_container.0.lock().await.get_data())
}


fn open_preferences_window(handle: &AppHandle) -> Result<(), tauri::Error> {
    if let Some(w) = handle.get_window("preferences") {
        w.unminimize()?;
        return w.set_focus();
    }

    WindowBuilder::new(
        handle,
        "preferences",
        WindowUrl::App("./src/window/window.html#preferences".into()),
    )
    .title(APP_NAME)
    .decorations(false)
    .inner_size(400.0, 500.0)
    .resizable(false)
    .visible(false)
    .build()?;

    Ok(())
}

fn open_profiles_window(handle: &AppHandle) -> Result<(), tauri::Error> {
    if let Some(w) = handle.get_window("profiles") {
        w.unminimize()?;
        return w.set_focus();
    }

    WindowBuilder::new(
        handle,
        "profiles",
        WindowUrl::App("./src/window/window.html#profiles".into()),
    )
    .title(APP_NAME)
    .decorations(false)
    .inner_size(400.0, 500.0)
    .resizable(false)
    .visible(false)
    .build()?;

    Ok(())
}

fn open_details_window(handle: &AppHandle, welcome: bool) -> Result<(), tauri::Error> {
    if let Some(w) = handle.get_window("details") {
        w.unminimize()?;
        return w.set_focus();
    }

    WindowBuilder::new(
        handle,
        "details",
        WindowUrl::App(
            format!(
                "./src/window/window.html{}#details",
                if welcome { "?welcome" } else { "" }
            )
            .into(),
        ),
    )
    .title(APP_NAME)
    .decorations(false)
    .inner_size(600.0, 600.0)
    .resizable(false)
    .visible(false)
    .build()?;

    Ok(())
}

async fn activate(handle: &AppHandle) -> Result<(), tauri::Error> {
    let config_container = handle.state::<ConfigContainer>();
    let lock = config_container.0.lock().await;

    if lock.get_profiles().selected_profile.is_none() {
        open_profiles_window(&handle)
    } else {
        open_details_window(&handle, false)
    }
}

async fn pipe_loop(handle: AppHandle, mut pipe_server: NamedPipeServer) -> io::Result<()> {
    loop {
        pipe_server.connect().await?;
        pipe_server = ServerOptions::new().create(NAMED_PIPE)?;
        pipe_server.disconnect()?;

        let _ = activate(&handle).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    let pipe_server = match ServerOptions::new()
        .first_pipe_instance(true)
        .create(NAMED_PIPE)
    {
        Ok(s) => s,
        Err(_) => {
            ClientOptions::new().open(NAMED_PIPE)?;
            return Ok(());
        }
    };

    tauri::async_runtime::set(tokio::runtime::Handle::current());


    let cache_manager = match CacheManager::load().await {
        Ok(cache) => {
            #[cfg(debug_assertions)]
            println!("✅ Cache: Successfully loaded cache manager");
            cache
        },
        Err(_e) => {
            #[cfg(debug_assertions)]
            println!("⚠️ Cache: Failed to load cache, creating new: {}", _e);
            CacheManager::new()
        }
    };
    
    tauri::Builder::new()
        .manage(ConfigContainer(Mutex::new(ConfigManager::load()?)))
        .manage(CacheContainer(Mutex::new(cache_manager)))
        .manage(Api::default())
        .manage(PlayerDataPollerContainer::default())
        .manage(OverlayPollerHandle::default())
        .system_tray(
            SystemTray::new().with_menu(
                SystemTrayMenu::new()
                    .add_item(
                        CustomMenuItem::new("version_info", format!("{APP_NAME} v{}", APP_VER))
                            .disabled(),
                    )
                    .add_native_item(SystemTrayMenuItem::Separator)
                    .add_item(CustomMenuItem::new("preferences", "Preferences"))
                    .add_item(CustomMenuItem::new("set_profile", "Set profile"))
                    .add_native_item(SystemTrayMenuItem::Separator)
                    .add_item(CustomMenuItem::new("exit", "Exit")),
            ),
        )
        .on_system_tray_event(|handle, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
                    "exit" => handle.exit(0),
                    "set_profile" => { let _ = open_profiles_window(&handle); }
                    "preferences" => { let _ = open_preferences_window(&handle); }
                    _ => (),
                }
            } else if let SystemTrayEvent::LeftClick { .. } = event {
                let handle_clone = handle.clone();
                async_runtime::spawn(async move { let _ = activate(&handle_clone).await; });
            }
        })
        .invoke_handler(tauri::generate_handler![
            open_preferences,
            open_profiles,
            get_preferences,
            set_preferences,
            get_profiles,
            set_profiles,
            get_profile_info,
            get_activity_info,
            search_profile,
            get_playerdata,
        ])
        .setup(|app| {
            let handle = app.handle();
            let pipe_handle = handle.clone();

            async_runtime::spawn(async move { pipe_loop(pipe_handle, pipe_server).await });

            async_runtime::spawn(async move {
                let config_container = handle.state::<ConfigContainer>();
                let lock = config_container.0.lock().await;

                if lock.get_profiles().selected_profile.is_none() {
                    let _ = open_profiles_window(&handle);
                } else {
                    if lock.get_preferences().enable_overlay {
                        let _ = create_overlay(handle.clone()).await;
                    }

                    let _ = open_details_window(&handle, false);
                }

                let poller_container = handle.state::<PlayerDataPollerContainer>();

                poller_container.0.lock().await.reset(handle.clone()).await;
            });

            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(|_, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => (),
        });

    Ok(())
}
