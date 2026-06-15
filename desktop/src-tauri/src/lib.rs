mod app;
mod audio;
mod auth;
mod import;
mod network;
mod shared;
mod track_cache;

mod discord;

use std::sync::{Arc, Mutex};
use tauri::Manager;

use network::server::ServerState;

#[cfg(not(target_os = "android"))]
use discord::DiscordState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // single-instance: только на десктопе
    #[cfg(not(target_os = "android"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }));
    }

    builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol("scproxy", |_ctx, request, responder| {
            let Some(state) = network::proxy::STATE.get() else {
                responder.respond(
                    http::Response::builder()
                        .status(503)
                        .body(b"not ready".to_vec())
                        .unwrap(),
                );
                return;
            };
            state.rt_handle.spawn(async move {
                responder.respond(network::proxy::handle_uri(request).await);
            });
        })
        .setup(move |app| {
            let cache_dir = app
                .path()
                .app_cache_dir()
                .expect("failed to resolve app cache dir");
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let audio_dir = cache_dir.join("audio");
            std::fs::create_dir_all(&audio_dir).ok();

            let liked_audio_dir = cache_dir.join("audio_liked");
            std::fs::create_dir_all(&liked_audio_dir).ok();

            let incoming_audio_dir = cache_dir.join("audio_incoming");
            std::fs::create_dir_all(&incoming_audio_dir).ok();

            let assets_dir = cache_dir.join("assets");
            std::fs::create_dir_all(&assets_dir).ok();

            let wallpapers_dir = cache_dir.join("wallpapers");
            std::fs::create_dir_all(&wallpapers_dir).ok();

            let images_dir = data_dir.join("images");
            std::fs::create_dir_all(&images_dir).ok();

            let http_client = reqwest::Client::new();
            let auth_http_client = http_client.clone();
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

            network::proxy::STATE
                .set(network::proxy::State {
                    assets_dir,
                    http_client: http_client.clone(),
                    rt_handle: rt.handle().clone(),
                })
                .ok();

            network::image_cache::STATE
                .set(network::image_cache::ImageCache {
                    dir: images_dir,
                    http_client,
                })
                .ok();

            let (static_port, proxy_port) = rt.block_on(network::server::start_all(wallpapers_dir));
            let rt_handle = rt.handle().clone();

            // ─── DPI desync ────────────────────────────────────────────────────────────
            // ИСПРАВЛЕНИЕ: На Android probe_in_background() может блокировать поток на
            // сетевых системных вызовах, недоступных без root. Оборачиваем полностью.
            // На Android DPI-bypass всё равно не нужен (нет TCP-манипуляций в юзерспейсе).
            #[cfg(not(target_os = "android"))]
            if let Ok(d) = rt.block_on(dpi_desync::Desync::spawn(true)) {
                network::dpi::install(d);
                rt.block_on(network::dpi::probe_in_background());
            }

            // Держим runtime живым в фоновом потоке
            std::thread::spawn(move || {
                rt.block_on(std::future::pending::<()>());
            });

            app.manage(Arc::new(ServerState {
                static_port,
                proxy_port,
            }));

            app::diagnostics::mark_session_started(app.handle());
            app::diagnostics::start_linux_fd_monitor(app.handle());

            // Discord: только на десктопе
            #[cfg(not(target_os = "android"))]
            app.manage(Arc::new(DiscordState {
                client: Mutex::new(None),
            }));

            // FFmpeg + track_cache: только на десктопе
            #[cfg(not(target_os = "android"))]
            {
                let ffmpeg_dir = cache_dir.join("ffmpeg");
                std::fs::create_dir_all(&ffmpeg_dir).ok();

                let mut track_cache_state =
                    track_cache::init(audio_dir, liked_audio_dir, incoming_audio_dir);
                track_cache_state.set_app_handle(app.handle().clone());
                let recovery_state = track_cache_state.clone();
                app.manage(track_cache_state);
                rt_handle.spawn(async move {
                    recovery_state.init_ffmpeg(ffmpeg_dir).await;
                    recovery_state.recover_incoming().await;
                });
            }

            // Android: track_cache без ffmpeg
            #[cfg(target_os = "android")]
            {
                let mut track_cache_state =
                    track_cache::init(audio_dir, liked_audio_dir, incoming_audio_dir);
                track_cache_state.set_app_handle(app.handle().clone());
                app.manage(track_cache_state);
            }

            let audio_state = audio::init();
            let analyser_buffer = audio_state.analyser_buffer.clone();
            app.manage(audio_state);
            audio::start_tick_emitter(app.handle());
            audio::start_media_controls(app.handle());
            audio::start_default_output_monitor(app.handle());
            audio::start_fft_thread(app.handle().clone(), analyser_buffer);

            // Tray и popover: только на десктопе
            #[cfg(not(target_os = "android"))]
            {
                app.manage(app::popover::TrayState::default());
                app::tray::setup_tray(app).expect("failed to setup tray");
            }

            let auth_state =
                auth::SessionStore::init(data_dir.clone(), auth_http_client, rt_handle.clone());
            app.manage(auth_state);

            // ─── Call client ───────────────────────────────────────────────────────────
            // ИСПРАВЛЕНИЕ: call-client использует нативные сокеты и может вызывать
            // maybe_autostart(), который спавнит фоновый процесс — запрещено на Android.
            // CallState не регистрируется, значит call_set_enabled/call_status не будут
            // доступны — они исключены из invoke_handler ниже через #[cfg].
            #[cfg(not(target_os = "android"))]
            {
                let call_state = network::call::CallState::init(data_dir.clone(), rt_handle);
                network::call::manage_state(app.handle(), call_state.clone());
                network::call::maybe_autostart(app.handle(), call_state);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Десктоп: скрываем в трей вместо закрытия
            #[cfg(not(target_os = "android"))]
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            #[cfg(not(target_os = "android"))]
            tauri::WindowEvent::Focused(false)
                if window.label() == app::popover::LABEL =>
            {
                let st = window.app_handle().state::<app::popover::TrayState>();
                if !st.is_pinned() {
                    let _ = window.hide();
                    st.mark_hidden();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            network::server::get_server_ports,
            app::diagnostics::diagnostics_log,
            // Discord: есть android-стабы, работает везде
            discord::discord_connect,
            discord::discord_disconnect,
            discord::discord_set_activity,
            discord::discord_clear_activity,
            // Audio: cpal/rodio поддерживают Android через AAudio (minSdk 26)
            audio::audio_load_file,
            audio::audio_load_url,
            audio::audio_play,
            audio::audio_pause,
            audio::audio_stop,
            audio::audio_seek,
            audio::audio_set_volume,
            audio::audio_set_playback_rate,
            audio::audio_set_ab_loop,
            audio::audio_get_position,
            audio::audio_set_eq,
            audio::audio_set_normalization,
            audio::audio_is_playing,
            audio::audio_set_metadata,
            audio::audio_set_playback_state,
            audio::audio_set_media_position,
            audio::audio_list_devices,
            audio::audio_switch_device,
            audio::audio_set_follow_default_output,
            audio::audio_set_lyrics_timeline,
            audio::audio_clear_lyrics_timeline,
            audio::audio_set_comments_timeline,
            audio::audio_clear_comments_timeline,
            audio::audio_preview_play,
            audio::audio_preview_stop,
            audio::save_track_to_path,
            // ИСПРАВЛЕНИЕ: import через subprocess — только десктоп
            #[cfg(not(target_os = "android"))]
            import::ym_import_start,
            #[cfg(not(target_os = "android"))]
            import::ym_import_stop,
            // Track cache: работает на обеих платформах
            track_cache::track_ensure_cached,
            track_cache::track_export,
            track_cache::track_is_cached,
            track_cache::track_transcode_status,
            track_cache::track_get_cache_path,
            track_cache::track_get_cache_info,
            track_cache::track_preload,
            track_cache::track_cache_size,
            track_cache::track_liked_cache_size,
            track_cache::track_clear_cache,
            track_cache::track_clear_liked_cache,
            track_cache::track_remove_cached,
            track_cache::track_list_cached,
            track_cache::track_cache_inventory,
            track_cache::track_enforce_cache_limit,
            track_cache::track_cache_likes,
            track_cache::track_cache_likes_running,
            track_cache::track_cancel_cache_likes,
            network::image_cache::image_cache_size,
            network::image_cache::image_cache_clear,
            // ИСПРАВЛЕНИЕ: call команды требуют CallState в manage() — только десктоп.
            // На Android CallState не регистрируется → вызов этих команд вызвал бы panic.
            #[cfg(not(target_os = "android"))]
            network::call::call_set_enabled,
            #[cfg(not(target_os = "android"))]
            network::call::call_is_enabled,
            #[cfg(not(target_os = "android"))]
            network::call::call_status,
            auth::auth_status,
            auth::auth_set_session,
            auth::auth_logout,
            auth::auth_set_premium,
            // DPI команды безопасны на Android: без dpi_desync::install() они просто
            // возвращают дефолтные значения через DESYNC.get() == None
            network::dpi::dpi_set_enabled,
            network::dpi::dpi_is_enabled,
            network::dpi::dpi_strategy,
            network::wallpapers::wallpaper_search,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
