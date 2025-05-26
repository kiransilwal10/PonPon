// src-tauri/src/lib.rs
use cfg_if::cfg_if;
use serde::Serialize;
use tauri::{Emitter, Manager}; // Removed Window as it's not used in command signatures for this iteration

// Conditional imports for Windows
cfg_if! {
    if #[cfg(windows)] {
        use windows::{
            core::{Result as WinCoreResult, HSTRING},
            Media::Control::{
                GlobalSystemMediaTransportControlsSessionManager,
                GlobalSystemMediaTransportControlsSession, // Keep as it's used for type of `session`
                GlobalSystemMediaTransportControlsSessionPlaybackStatus,
                // GlobalSystemMediaTransportControlsSessionTimelineProperties, // Not explicitly named, but methods are on session
            },
            Win32::System::WinRT::{RoInitialize, /* RoUninitialize, (can be removed if thread handles its own) */ RO_INIT_TYPE},
            // Foundation::IAsyncOperation, // Implicitly used by .get()
        };
    }
}

// Conditional imports for Linux
cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
        use zbus::{Connection, Proxy, zvariant::{Value, Dict, EncodingContext}};
        use std::collections::HashMap;
        use std::convert::TryInto;
    }
}

#[derive(Clone, Serialize, Debug, Default)]
struct MediaInfo {
    title: Option<String>,
    artist: Option<String>,
    is_playing: bool,
    album_art_url: Option<String>,
    current_time_ms: Option<u64>,
    total_time_ms: Option<u64>,
}

#[cfg(windows)]
fn ensure_winrt_initialized_on_thread() -> WinCoreResult<()> {
    unsafe {
        RoInitialize(RO_INIT_TYPE(1))?;
        Ok(())
    }
}

#[cfg(windows)]
fn get_windows_smtc_media_info() -> WinCoreResult<MediaInfo> {
    // ensure_winrt_initialized_on_thread is called by the polling thread once.

    let manager_op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?;
    let manager = manager_op.get()?;

    match manager.GetCurrentSession() {
        Ok(session) => {
            match session.TryGetMediaPropertiesAsync() {
                Ok(properties_op) => match properties_op.get() {
                    Ok(properties) => match session.GetPlaybackInfo() {
                        Ok(playback_info) => {
                            let title = properties.Title().map(|h: HSTRING| h.to_string()).ok();
                            let artist = properties.Artist().map(|h: HSTRING| h.to_string()).ok();
                            let mut album_art_url: Option<String> = None;
                            // TODO: Implement actual album art fetching from properties.Thumbnail()
                            // This involves reading IRandomAccessStreamReference, async operations, and data conversion.
                            // For now: album_art_url = Some("placeholder_for_win_art.png".to_string());

                            let mut current_time_ms: Option<u64> = None;
                            let mut total_time_ms: Option<u64> = None;
                            if let Ok(timeline_props) = session.GetTimelineProperties() {
                                if let Ok(pos) = timeline_props.Position() {
                                    // Duration is i64 (100-nanosecond ticks). Convert to u64 ms.
                                    // Ensure non-negative before converting to u64.
                                    if pos.Duration >= 0 {
                                        current_time_ms = Some((pos.Duration / 10000) as u64);
                                    }
                                }
                                if let Ok(end) = timeline_props.EndTime() {
                                    if end.Duration >= 0 {
                                        total_time_ms = Some((end.Duration / 10000) as u64);
                                    }
                                }
                            }

                            match playback_info.PlaybackStatus() {
                                Ok(status) => {
                                    let is_playing = status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
                                    if title.is_none() && artist.is_none() && !is_playing {
                                        // Only default if truly idle
                                        return Ok(MediaInfo::default());
                                    }
                                    Ok(MediaInfo {
                                        title,
                                        artist,
                                        is_playing,
                                        album_art_url,
                                        current_time_ms,
                                        total_time_ms,
                                    })
                                }
                                Err(_) => Ok(MediaInfo {
                                    album_art_url,
                                    current_time_ms,
                                    total_time_ms,
                                    ..Default::default()
                                }), // Keep whatever info we got
                            }
                        }
                        Err(_) => Ok(MediaInfo {
                            album_art_url: None,
                            current_time_ms: None,
                            total_time_ms: None,
                            ..Default::default()
                        }),
                    },
                    Err(_) => Ok(MediaInfo::default()),
                },
                Err(_) => Ok(MediaInfo::default()),
            }
        }
        Err(_) => Ok(MediaInfo::default()),
    }
}

#[cfg(all(target_os = "linux", feature = "with-zbus"))]
fn get_mpris_media_info_with_zbus() -> Result<MediaInfo, Box<dyn std::error::Error>> {
    let connection = Connection::session()?;
    let services = connection.list_names()?;
    let mpris_service_name = services
        .iter()
        .find(|name| name.as_str().starts_with("org.mpris.MediaPlayer2."));

    if let Some(service_name) = mpris_service_name {
        let player_proxy = Proxy::new(
            &connection,
            service_name.as_str(),
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        )?;

        let metadata_value: Value = player_proxy.get_property("Metadata")?;
        let playback_status_value: Value = player_proxy.get_property("PlaybackStatus")?;
        let position_us: i64 = player_proxy.get_property("Position")?; // Position is typically i64 for MPRIS

        let mut title = None;
        let mut artist = None;
        let mut album_art_url = None;
        let mut total_time_ms = None;

        let current_time_ms = if position_us >= 0 {
            Some((position_us / 1000) as u64)
        } else {
            None
        };

        if let Value::Dict(metadata_dict_ref) = metadata_value {
            let metadata_dict: &Dict = metadata_dict_ref;
            match metadata_dict.try_into() {
                Ok(metadata_map_val) => {
                    let metadata_map: HashMap<String, Value> = metadata_map_val;
                    if let Some(Value::Str(t)) = metadata_map.get("xesam:title") {
                        title = Some(t.to_string());
                    }
                    if let Some(Value::Array(artists_array)) = metadata_map.get("xesam:artist") {
                        if let Some(Value::Str(a)) = artists_array.get(0) {
                            artist = Some(a.to_string());
                        }
                    }
                    if let Some(Value::Str(art_url)) = metadata_map.get("mpris:artUrl") {
                        album_art_url = Some(art_url.to_string());
                    }
                    // mpris:length is in microseconds (i64 or u64)
                    if let Some(Value::I64(length_us)) = metadata_map.get("mpris:length") {
                        if *length_us >= 0 {
                            total_time_ms = Some((*length_us / 1000) as u64);
                        }
                    } else if let Some(Value::U64(length_us)) = metadata_map.get("mpris:length") {
                        total_time_ms = Some(*length_us / 1000);
                    }
                }
                Err(e) => eprintln!(
                    "Linux MPRIS: Failed to convert metadata Dict to HashMap: {}",
                    e
                ),
            }
        }
        let is_playing = if let Value::Str(status) = playback_status_value {
            status.as_str() == "Playing"
        } else {
            false
        };

        if title.is_none() && artist.is_none() && !is_playing {
            return Ok(MediaInfo::default());
        }
        Ok(MediaInfo {
            title,
            artist,
            is_playing,
            album_art_url,
            current_time_ms,
            total_time_ms,
        })
    } else {
        Ok(MediaInfo::default())
    }
}

fn get_current_os_media_info() -> MediaInfo {
    // ... (This function remains the same, calling the platform-specific ones)
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            #[cfg(feature = "with-zbus")] {
                match get_mpris_media_info_with_zbus() {
                    Ok(info) => info,
                    Err(e) => { eprintln!("Linux: Failed to get MPRIS info: {}", e); MediaInfo::default() }
                }
            } #[cfg(not(feature = "with-zbus"))] {
                 MediaInfo { title: Some("Linux: zbus N/A".into()), ..Default::default() }
            }
        } else if #[cfg(windows)] {
            match get_windows_smtc_media_info() {
                Ok(info) => info,
                Err(e) => {
                    let emsg = format!("{:?}", e); eprintln!("Windows: Failed to get SMTC info: {}", emsg);
                    MediaInfo { title: Some("Windows: Error".into()), artist: Some(emsg.chars().take(100).collect()), ..Default::default() }
                }
            }
        } else if #[cfg(target_os = "macos")] {
            // Placeholder for macOS, eventually replace with actual call
             let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
             let total_sim_ms = 240 * 1000; let current_sim_ms = (now % 240) * 1000;
             MediaInfo {
                title: Some(format!("macOS Track {}", now % 2)), artist: Some("macOS Artist".into()),
                is_playing: now % 20 < 10, album_art_url: Some("https://via.placeholder.com/40".to_string()),
                current_time_ms: Some(current_sim_ms), total_time_ms: Some(total_sim_ms),
             }
        } else {
             let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
             let total_sim_ms = 240 * 1000; let current_sim_ms = (now % 240) * 1000;
             MediaInfo {
                title: Some(format!("Other OS Track {}", now % 2)), artist: Some("Generic Artist".into()),
                is_playing: now % 20 < 10, album_art_url: Some("https://via.placeholder.com/40".to_string()),
                current_time_ms: Some(current_sim_ms), total_time_ms: Some(total_sim_ms),
             }
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// --- Media Control Commands ---
#[tauri::command]
fn system_media_toggle_play_pause() {
    println!("Rust: Command system_media_toggle_play_pause received.");
    cfg_if! {
        if #[cfg(windows)] {
            if let Ok(manager_op) = windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                if let Ok(manager) = manager_op.get() {
                    if let Ok(session) = manager.GetCurrentSession() {
                        if let Err(e) = session.TryTogglePlayPauseAsync().map(|op| op.get()) {
                            eprintln!("Windows SMTC: Failed to toggle play/pause: {:?}", e);
                        }
                    }
                }
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            if let Ok(connection) = Connection::session() {
                if let Ok(services) = connection.list_names() {
                    if let Some(service_name) = services.iter().find(|name| name.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&connection, service_name.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                            if let Err(e) = proxy.call_method("PlayPause", &()) {
                                eprintln!("Linux MPRIS: Failed to call PlayPause: {}", e);
                            }
                        }
                    }
                }
            }
        } else {
            println!("system_media_toggle_play_pause: Not implemented for this OS or zbus feature disabled.");
        }
    }
}

#[tauri::command]
fn system_media_next_track() {
    println!("Rust: Command system_media_next_track received.");
    cfg_if! {
        if #[cfg(windows)] {
            if let Ok(manager_op) = windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                if let Ok(manager) = manager_op.get() {
                    if let Ok(session) = manager.GetCurrentSession() {
                       if let Err(e) = session.TrySkipNextAsync().map(|op| op.get()) {
                           eprintln!("Windows SMTC: Failed to skip next: {:?}", e);
                       }
                    }
                }
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
             if let Ok(connection) = Connection::session() {
                if let Ok(services) = connection.list_names() {
                    if let Some(service_name) = services.iter().find(|name| name.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&connection, service_name.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                            if let Err(e) = proxy.call_method("Next", &()) {
                                eprintln!("Linux MPRIS: Failed to call Next: {}", e);
                            }
                        }
                    }
                }
            }
        } else {
            println!("system_media_next_track: Not implemented for this OS or zbus feature disabled.");
        }
    }
}

#[tauri::command]
fn system_media_previous_track() {
    println!("Rust: Command system_media_previous_track received.");
    cfg_if! {
        if #[cfg(windows)] {
            if let Ok(manager_op) = windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                if let Ok(manager) = manager_op.get() {
                    if let Ok(session) = manager.GetCurrentSession() {
                        if let Err(e) = session.TrySkipPreviousAsync().map(|op| op.get()) {
                            eprintln!("Windows SMTC: Failed to skip previous: {:?}", e);
                        }
                    }
                }
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            if let Ok(connection) = Connection::session() {
                if let Ok(services) = connection.list_names() {
                    if let Some(service_name) = services.iter().find(|name| name.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&connection, service_name.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                           if let Err(e) = proxy.call_method("Previous", &()) {
                               eprintln!("Linux MPRIS: Failed to call Previous: {}", e);
                           }
                        }
                    }
                }
            }
        } else {
            println!("system_media_previous_track: Not implemented for this OS or zbus feature disabled.");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    builder = builder.setup(|app| {
        let main_window = app.get_webview_window("main").unwrap(); 
        std::thread::spawn(move || {
            cfg_if! {
                if #[cfg(windows)] {
                    if let Err(e) = ensure_winrt_initialized_on_thread() {
                        eprintln!("Failed to initialize WinRT in media polling thread. SMTC might not work: {:?}", e);
                    }
                }
            }
            loop {
                let media_info = get_current_os_media_info();
                main_window.emit("media-info-update", &media_info).unwrap_or_else(|e| {
                    eprintln!("Failed to emit media-info-update: {}", e);
                });
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
        Ok(())
    });

    builder
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            system_media_toggle_play_pause,
            system_media_next_track,
            system_media_previous_track
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
