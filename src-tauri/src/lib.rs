// src-tauri/src/lib.rs
use cfg_if::cfg_if;
use serde::Serialize;
use tauri::{Emitter, Manager};

// Conditional imports for Windows
cfg_if! {
    if #[cfg(windows)] {
        use windows::{
            core::{Result as WinCoreResult, HSTRING},
            Media::Control::{
                GlobalSystemMediaTransportControlsSessionManager,
                GlobalSystemMediaTransportControlsSession,
                GlobalSystemMediaTransportControlsSessionPlaybackInfo, // Used to get PlaybackControls
                GlobalSystemMediaTransportControlsSessionPlaybackControls, // CORRECTED IMPORT: For IsNextEnabled, IsPreviousEnabled
                GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            },
            Win32::System::WinRT::{RoInitialize, RO_INIT_TYPE},
            // Foundation::IAsyncAction, // .get() on IAsyncAction is often available via Foundation feature without direct import
        };
    }
}

// Conditional imports for Linux
cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
        use zbus::{Connection, Proxy, zvariant::{Value, Dict}};
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
fn get_current_smtc_session() -> WinCoreResult<GlobalSystemMediaTransportControlsSession> {
    let manager_op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?;
    let manager = manager_op.get()?;
    manager.GetCurrentSession()
}

#[cfg(windows)]
fn get_windows_smtc_media_info() -> WinCoreResult<MediaInfo> {
    // ... (This function's internal logic remains the same as the last version,
    // as it doesn't directly use IsNextEnabled/IsPreviousEnabled for data fetching)
    match get_current_smtc_session() {
        Ok(session) => match session.TryGetMediaPropertiesAsync() {
            Ok(properties_op) => match properties_op.get() {
                Ok(properties) => match session.GetPlaybackInfo() {
                    Ok(playback_info) => {
                        let title = properties.Title().map(|h: HSTRING| h.to_string()).ok();
                        let artist = properties.Artist().map(|h: HSTRING| h.to_string()).ok();
                        let album_art_url: Option<String> = None;

                        let mut current_time_ms: Option<u64> = None;
                        let mut total_time_ms: Option<u64> = None;
                        if let Ok(timeline_props) = session.GetTimelineProperties() {
                            if let Ok(pos) = timeline_props.Position() {
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
                            }),
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
        },
        Err(_) => Ok(MediaInfo::default()),
    }
}

#[cfg(all(target_os = "linux", feature = "with-zbus"))]
fn get_mpris_media_info_with_zbus() -> Result<MediaInfo, Box<dyn std::error::Error>> {
    // ... (This function remains the same as the last correct version)
    let connection = Connection::session()?;
    let services = connection.list_names()?;
    let service_name_option = services
        .iter()
        .find(|name| name.as_str().starts_with("org.mpris.MediaPlayer2."));

    if let Some(service_name) = service_name_option {
        let player_proxy = Proxy::new(
            &connection,
            service_name.as_str(),
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        )?;

        let metadata_value: Value = player_proxy.get_property("Metadata")?;
        let playback_status_value: Value = player_proxy.get_property("PlaybackStatus")?;
        let position_us: i64 = match player_proxy.get_property("Position") {
            Ok(Value::I64(p)) => p,
            Ok(Value::U64(p)) => p as i64,
            _ => 0,
        };

        let current_time_ms = if position_us >= 0 {
            Some((position_us / 1000) as u64)
        } else {
            None
        };
        let mut title = None;
        let mut artist = None;
        let mut album_art_url = None;
        let mut total_time_ms = None;

        if let Value::Dict(dict_ref) = metadata_value {
            if let Ok(map) = (*dict_ref).try_into() as Result<HashMap<String, Value>, _> {
                if let Some(Value::Str(t)) = map.get("xesam:title") {
                    title = Some(t.to_string());
                }
                if let Some(Value::Array(arr)) = map.get("xesam:artist") {
                    if let Some(Value::Str(a)) = arr.get(0) {
                        artist = Some(a.to_string());
                    }
                }
                if let Some(Value::Str(u)) = map.get("mpris:artUrl") {
                    album_art_url = Some(u.to_string());
                }
                if let Some(Value::I64(len)) = map.get("mpris:length") {
                    if *len >= 0 {
                        total_time_ms = Some((*len / 1000) as u64);
                    }
                } else if let Some(Value::U64(len)) = map.get("mpris:length") {
                    total_time_ms = Some(*len / 1000);
                }
            } else {
                eprintln!(
                    "Linux MPRIS: Error converting metadata dict: {:?}",
                    dict_ref
                );
            }
        }
        let is_playing =
            matches!(playback_status_value, Value::Str(ref s) if s.as_str() == "Playing");
        if title.is_none() && artist.is_none() && !is_playing {
            return Ok(MediaInfo::default());
        }
        return Ok(MediaInfo {
            title,
            artist,
            is_playing,
            album_art_url,
            current_time_ms,
            total_time_ms,
        });
    }
    Ok(MediaInfo::default())
}

fn get_current_os_media_info() -> MediaInfo {
    // ... (This function remains the same)
    cfg_if! {
        if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            match get_mpris_media_info_with_zbus() {
                Ok(info) => info,
                Err(e) => { eprintln!("Linux MPRIS error: {}", e); MediaInfo::default() }
            }
        } else if #[cfg(windows)] {
            match get_windows_smtc_media_info() {
                Ok(info) => info,
                Err(e) => { eprintln!("Windows SMTC error: {:?}", e); MediaInfo::default() }
            }
        } else if #[cfg(target_os = "macos")] {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let sim_ms = (now % 240) * 1000;
            MediaInfo {
                title: Some(format!("macOS Track {}", now % 2)), artist: Some("macOS Artist".into()),
                is_playing: now % 20 < 10, album_art_url: Some("https://via.placeholder.com/40".into()),
                current_time_ms: Some(sim_ms), total_time_ms: Some(240 * 1000),
            }
        } else {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let sim_ms = (now % 240) * 1000;
            MediaInfo {
                title: Some(format!("Other OS Track {}", now % 2)), artist: Some("Generic Artist".into()),
                is_playing: now % 20 < 10, album_art_url: Some("https://via.placeholder.com/40".into()),
                current_time_ms: Some(sim_ms), total_time_ms: Some(240 * 1000),
            }
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn system_media_toggle_play_pause() {
    // ... (This command remains the same as the last correct version)
    println!("Rust: Command system_media_toggle_play_pause received.");
    cfg_if! {
        if #[cfg(windows)] {
            match get_current_smtc_session() {
                Ok(session) => {
                    match session.TryTogglePlayPauseAsync() {
                        Ok(op) => {
                            if let Err(e) = op.get() {
                                eprintln!("Windows SMTC: Failed to complete toggle play/pause op: {:?}", e);
                            } else {
                                println!("Windows SMTC: TogglePlayPause command sent successfully.");
                            }
                        }
                        Err(e) => eprintln!("Windows SMTC: TryTogglePlayPauseAsync call failed: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Windows SMTC: Could not get session for PlayPause: {:?}", e),
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            if let Ok(conn) = Connection::session() {
                if let Ok(names) = conn.list_names() {
                    if let Some(svc) = names.iter().find(|n| n.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&conn, svc.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                            match proxy.call_method("PlayPause", &()) {
                                Ok(_) => println!("Linux MPRIS: PlayPause command sent."),
                                Err(e) => eprintln!("Linux MPRIS: Failed to call PlayPause: {}", e),
                            }
                        } else { eprintln!("Linux MPRIS: Could not create proxy for PlayPause."); }
                    } else { eprintln!("Linux MPRIS: No player found for PlayPause."); }
                } else { eprintln!("Linux MPRIS: Could not list D-Bus names for PlayPause."); }
            } else { eprintln!("Linux MPRIS: Could not connect to D-Bus for PlayPause."); }
        } else {
            println!("system_media_toggle_play_pause: Not implemented for this OS/feature set.");
        }
    }
}

#[tauri::command]
fn system_media_next_track() {
    println!("Rust: Command system_media_next_track received.");
    cfg_if! {
        if #[cfg(windows)] {
            match get_current_smtc_session() {
                Ok(session) => {
                    match session.GetPlaybackInfo() { // PlaybackInfo contains Controls
                        Ok(playback_info) => {
                            // GetControls() returns Result<PlaybackControls>
                            match playback_info.Controls() {
                                Ok(controls) => { // controls is GlobalSystemMediaTransportControlsSessionPlaybackControls
                                    if controls.IsNextEnabled().unwrap_or(false) { // This should now work
                                        match session.TrySkipNextAsync() {
                                            Ok(op) => {
                                                if let Err(e) = op.get() {
                                                    eprintln!("Windows SMTC: Failed to complete skip next op: {:?}", e);
                                                } else {
                                                    println!("Windows SMTC: SkipNext command sent successfully.");
                                                }
                                            }
                                            Err(e) => eprintln!("Windows SMTC: TrySkipNextAsync call failed: {:?}", e),
                                        }
                                    } else {
                                        println!("Windows SMTC: Next track is not enabled by the current media session.");
                                    }
                                }
                                Err(e) => eprintln!("Windows SMTC: Could not get Controls from PlaybackInfo for NextTrack: {:?}", e),
                            }
                        }
                        Err(e) => eprintln!("Windows SMTC: Could not get PlaybackInfo for NextTrack: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Windows SMTC: Could not get session for NextTrack: {:?}", e),
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            // ... (Linux Next logic as before)
            if let Ok(conn) = Connection::session() {
                if let Ok(names) = conn.list_names() {
                    if let Some(svc) = names.iter().find(|n| n.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&conn, svc.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                            if proxy.get_property::<bool>("CanGoNext").unwrap_or(false) {
                                match proxy.call_method("Next", &()) {
                                    Ok(_) => println!("Linux MPRIS: Next command sent."),
                                    Err(e) => eprintln!("Linux MPRIS: Failed to call Next: {}", e),
                                }
                            } else {
                                println!("Linux MPRIS: Next track is not enabled (CanGoNext is false).");
                            }
                        } else { eprintln!("Linux MPRIS: Could not create proxy for Next."); }
                    } else { eprintln!("Linux MPRIS: No player found for Next."); }
                } else { eprintln!("Linux MPRIS: Could not list D-Bus names for Next."); }
            } else { eprintln!("Linux MPRIS: Could not connect to D-Bus for Next."); }
        } else if #[cfg(target_os = "macos")] {
            let _ = std::process::Command::new("osascript")
                .arg("-e").arg("tell application \"Spotify\" to next track").status();
            println!("macOS: Sent 'next track' to Spotify (result not checked).");
        } else {
            println!("system_media_next_track: Not implemented for this OS/feature set.");
        }
    }
}

#[tauri::command]
fn system_media_previous_track() {
    println!("Rust: Command system_media_previous_track received.");
    cfg_if! {
        if #[cfg(windows)] {
            match get_current_smtc_session() {
                Ok(session) => {
                    match session.GetPlaybackInfo() {
                        Ok(playback_info) => {
                            match playback_info.Controls() {
                                Ok(controls) => { // controls is GlobalSystemMediaTransportControlsSessionPlaybackControls
                                    if controls.IsPreviousEnabled().unwrap_or(false) { // This should now work
                                        match session.TrySkipPreviousAsync() {
                                            Ok(op) => {
                                                if let Err(e) = op.get() {
                                                    eprintln!("Windows SMTC: Failed to complete skip previous op: {:?}", e);
                                                } else {
                                                    println!("Windows SMTC: SkipPrevious command sent successfully.");
                                                }
                                            }
                                            Err(e) => eprintln!("Windows SMTC: TrySkipPreviousAsync call failed: {:?}", e),
                                        }
                                    } else {
                                        println!("Windows SMTC: Previous track is not enabled by the current media session.");
                                    }
                                }
                                Err(e) => eprintln!("Windows SMTC: Could not get Controls from PlaybackInfo for PreviousTrack: {:?}", e),
                            }
                        }
                        Err(e) => eprintln!("Windows SMTC: Could not get PlaybackInfo for PreviousTrack: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Windows SMTC: Could not get session for PreviousTrack: {:?}", e),
            }
        } else if #[cfg(all(target_os = "linux", feature = "with-zbus"))] {
            // ... (Linux Previous logic as before)
             if let Ok(conn) = Connection::session() {
                if let Ok(names) = conn.list_names() {
                    if let Some(svc) = names.iter().find(|n| n.as_str().starts_with("org.mpris.MediaPlayer2.")) {
                        if let Ok(proxy) = Proxy::new(&conn, svc.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player") {
                            if proxy.get_property::<bool>("CanGoPrevious").unwrap_or(false) {
                                match proxy.call_method("Previous", &()) {
                                    Ok(_) => println!("Linux MPRIS: Previous command sent."),
                                    Err(e) => eprintln!("Linux MPRIS: Failed to call Previous: {}", e),
                                }
                            } else {
                                println!("Linux MPRIS: Previous track is not enabled (CanGoPrevious is false).");
                            }
                        } else { eprintln!("Linux MPRIS: Could not create proxy for Previous."); }
                    } else { eprintln!("Linux MPRIS: No player found for Previous."); }
                } else { eprintln!("Linux MPRIS: Could not list D-Bus names for Previous."); }
            } else { eprintln!("Linux MPRIS: Could not connect to D-Bus for Previous."); }
        } else if #[cfg(target_os = "macos")] {
            let _ = std::process::Command::new("osascript")
                .arg("-e").arg("tell application \"Spotify\" to previous track").status();
            println!("macOS: Sent 'previous track' to Spotify (result not checked).");
        } else {
            println!("system_media_previous_track: Not implemented for this OS/feature set.");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    builder = builder.setup(|app| {
        let main_window = app.get_webview_window("main").unwrap();
        std::thread::spawn(move || {
            #[cfg(windows)]
            if let Err(e) = ensure_winrt_initialized_on_thread() {
                eprintln!("WinRT init failed in polling thread: {:?}", e);
            }

            loop {
                let info = get_current_os_media_info();
                if main_window.emit("media-info-update", &info).is_err() {
                    // eprintln!("Error emitting media-info-update");
                }
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
