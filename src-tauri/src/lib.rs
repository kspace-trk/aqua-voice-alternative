use base64::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> bool;
}

#[cfg(target_os = "macos")]
fn check_accessibility_permission() -> bool {
    unsafe {
        use core_foundation::base::TCFType;
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::string::CFString;
        use core_foundation::boolean::CFBoolean;

        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::true_value();
        
        let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
        
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
    }
}

// Application state
struct AppState {
    current_shortcut: Mutex<Option<Shortcut>>,
    audio_sender: Mutex<Option<mpsc::Sender<AudioCommand>>>,
    api_key: Mutex<String>,
    model: Mutex<String>,
    tray_icon: Mutex<Option<TrayIcon>>,
}

enum AudioCommand {
    StartRecording,
    StopRecording,
}

// Audio recording state
struct RecordingState {
    samples: Vec<f32>,
    is_recording: bool,
}

// Gemini API types
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[tauri::command]
fn execute_paste(_app: AppHandle) {
    use std::process::Command;

    let script = r#"
        tell application "System Events"
            keystroke "v" using command down
        end tell
    "#;

    let output = Command::new("osascript").arg("-e").arg(script).output();

    match output {
        Ok(o) => {
            if !o.status.success() {
                println!(
                    "Paste Script Error: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            } else {
                println!("Paste Script Success");
            }
        }
        Err(e) => println!("Failed to execute paste command: {}", e),
    }
}

#[tauri::command]
fn set_api_key(app: AppHandle, api_key: String) {
    let state = app.state::<AppState>();
    *state.api_key.lock().unwrap() = api_key;
    println!("API key updated");
}

#[tauri::command]
fn set_model(app: AppHandle, model: String) {
    let state = app.state::<AppState>();
    *state.model.lock().unwrap() = model;
    println!("Model updated");
}

#[tauri::command]
fn register_shortcut(app: AppHandle, shortcut_str: String) -> Result<(), String> {
    let state = app.state::<AppState>();

    // Unregister previous shortcut
    if let Some(old_shortcut) = state.current_shortcut.lock().unwrap().take() {
        let _ = app.global_shortcut().unregister(old_shortcut);
    }

    // Parse the shortcut string
    let shortcut = parse_shortcut(&shortcut_str).map_err(|e| e.to_string())?;

    // Get audio sender
    let sender = state.audio_sender.lock().unwrap().clone();

    // Register new shortcut
    app.global_shortcut()
        .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
            if let Some(ref tx) = sender {
                match event.state {
                    ShortcutState::Pressed => {
                        println!("Shortcut pressed - starting recording");
                        let _ = tx.blocking_send(AudioCommand::StartRecording);
                    }
                    ShortcutState::Released => {
                        println!("Shortcut released - stopping recording");
                        let _ = tx.blocking_send(AudioCommand::StopRecording);
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;

    // Store the shortcut
    *state.current_shortcut.lock().unwrap() = Some(shortcut);

    println!("Registered shortcut: {}", shortcut_str);
    Ok(())
}

fn parse_shortcut(s: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "COMMANDORCONTROL" | "CMD" | "COMMAND" | "CTRL" | "CONTROL" => {
                modifiers |= Modifiers::META;
            }
            "SHIFT" => {
                modifiers |= Modifiers::SHIFT;
            }
            "ALT" | "OPTION" => {
                modifiers |= Modifiers::ALT;
            }
            key => {
                code = Some(match key {
                    "SPACE" | " " => Code::Space,
                    "A" => Code::KeyA,
                    "B" => Code::KeyB,
                    "C" => Code::KeyC,
                    "D" => Code::KeyD,
                    "E" => Code::KeyE,
                    "F" => Code::KeyF,
                    "G" => Code::KeyG,
                    "H" => Code::KeyH,
                    "I" => Code::KeyI,
                    "J" => Code::KeyJ,
                    "K" => Code::KeyK,
                    "L" => Code::KeyL,
                    "M" => Code::KeyM,
                    "N" => Code::KeyN,
                    "O" => Code::KeyO,
                    "P" => Code::KeyP,
                    "Q" => Code::KeyQ,
                    "R" => Code::KeyR,
                    "S" => Code::KeyS,
                    "T" => Code::KeyT,
                    "U" => Code::KeyU,
                    "V" => Code::KeyV,
                    "W" => Code::KeyW,
                    "X" => Code::KeyX,
                    "Y" => Code::KeyY,
                    "Z" => Code::KeyZ,
                    "0" => Code::Digit0,
                    "1" => Code::Digit1,
                    "2" => Code::Digit2,
                    "3" => Code::Digit3,
                    "4" => Code::Digit4,
                    "5" => Code::Digit5,
                    "6" => Code::Digit6,
                    "7" => Code::Digit7,
                    "8" => Code::Digit8,
                    "9" => Code::Digit9,
                    "HOME" => Code::Home,
                    "END" => Code::End,
                    "PAGEUP" => Code::PageUp,
                    "PAGEDOWN" => Code::PageDown,
                    "UP" => Code::ArrowUp,
                    "DOWN" => Code::ArrowDown,
                    "LEFT" => Code::ArrowLeft,
                    "RIGHT" => Code::ArrowRight,
                    "ENTER" => Code::Enter,
                    "ESCAPE" => Code::Escape,
                    "TAB" => Code::Tab,
                    "BACKSPACE" => Code::Backspace,
                    "DELETE" => Code::Delete,
                    _ => return Err(format!("Unknown key: {}", key)),
                });
            }
        }
    }

    let code = code.ok_or("No key specified")?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn update_tray_status(app: &AppHandle, status: &str) {
    let state = app.state::<AppState>();
    let tray_lock = state.tray_icon.lock().unwrap();
    if let Some(tray) = tray_lock.as_ref() {
        let (tooltip, title) = match status {
            "recording" => ("AquaVoice - Recording...", "🎙️"),
            "processing" => ("AquaVoice - Processing...", "⏳"),
            "transcribing" => ("AquaVoice - Transcribing...", "🔄"),
            "success" => ("AquaVoice - Done", "✅"),
            "error" => ("AquaVoice - Error", "❌"),
            _ => ("AquaVoice - Ready", ""),
        };
        println!("Updating tray status to: {} ({})", status, title);
        let _ = tray.set_tooltip(Some(tooltip));
        let _ = tray.set_title(Some(title));
    } else {
        println!("Warning: Tray icon not available");
    }
}

fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer =
            WavWriter::new(&mut cursor, spec).map_err(|e| format!("WAV writer error: {}", e))?;

        for &sample in samples {
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Write sample error: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("Finalize error: {}", e))?;
    }

    Ok(cursor.into_inner())
}

async fn transcribe_with_gemini(api_key: &str, model: &str, audio_data: &[u8]) -> Result<String, String> {
    let base64_audio = base64::engine::general_purpose::STANDARD.encode(audio_data);

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: "audio/wav".to_string(),
                        data: base64_audio,
                    },
                },
                Part::Text {
                    text: "これは、PC作業時の音声入力のための音声です。音声を文字起こししてください。音声の内容のみを出力し、余計な説明は不要です。"
                        .to_string(),
                },
            ],
        }],
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error: {}", error_text));
    }

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let text = gemini_response
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .unwrap_or_default();

    Ok(text.trim().to_string())
}

fn start_audio_processing(app: AppHandle, mut rx: mpsc::Receiver<AudioCommand>) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No input device available");

        let config = device.default_input_config().expect("No default config");
        let sample_rate = config.sample_rate().0;

        println!("Using audio device: {}", device.name().unwrap_or_default());
        println!("Sample rate: {}", sample_rate);

        let recording_state = Arc::new(Mutex::new(RecordingState {
            samples: Vec::new(),
            is_recording: false,
        }));

        let recording_state_clone = Arc::clone(&recording_state);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut state = recording_state_clone.lock().unwrap();
                    if state.is_recording {
                        state.samples.extend_from_slice(data);
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .expect("Failed to build input stream");

        stream.play().expect("Failed to start stream");

        let rt = tokio::runtime::Runtime::new().unwrap();

        loop {
            match rx.blocking_recv() {
                Some(AudioCommand::StartRecording) => {
                    println!("Starting recording...");
                    let mut state = recording_state.lock().unwrap();
                    state.samples.clear();
                    state.is_recording = true;
                    update_tray_status(&app, "recording");
                    let _ = app.emit("status-changed", "recording");
                }
                Some(AudioCommand::StopRecording) => {
                    println!("Stopping recording...");
                    update_tray_status(&app, "processing");
                    let _ = app.emit("status-changed", "processing");
                    let samples: Vec<f32>;
                    {
                        let mut state = recording_state.lock().unwrap();
                        state.is_recording = false;
                        samples = state.samples.clone();
                    }

                    if samples.is_empty() {
                        println!("No audio recorded");
                        update_tray_status(&app, "error");
                        let _ = app.emit("status-changed", "error:No audio recorded");
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        update_tray_status(&app, "idle");
                        continue;
                    }

                    println!("Recorded {} samples", samples.len());

                    // Convert to WAV
                    let wav_data = match samples_to_wav(&samples, sample_rate) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("WAV conversion error: {}", e);
                            continue;
                        }
                    };

                    println!("WAV data size: {} bytes", wav_data.len());

                    // Get API key and model
                    let (api_key, model): (String, String) = {
                        let state = app.state::<AppState>();
                        let api_key = state.api_key.lock().unwrap().clone();
                        let model = state.model.lock().unwrap().clone();
                        (api_key, model)
                    };

                    if api_key.is_empty() {
                        eprintln!("No API key set");
                        continue;
                    }

                    if model.is_empty() {
                        eprintln!("No model set");
                        continue;
                    }

                    // Transcribe with Gemini
                    let app_clone = app.clone();
                    update_tray_status(&app, "transcribing");
                    let _ = app.emit("status-changed", "transcribing");
                    rt.block_on(async {
                        match transcribe_with_gemini(&api_key, &model, &wav_data).await {
                            Ok(text) => {
                                println!("Transcription result: {}", text);

                                if !text.is_empty() {
                                    // Copy to clipboard
                                    if let Err(e) =
                                        app_clone.clipboard().write_text(text.clone())
                                    {
                                        eprintln!("Clipboard error: {}", e);
                                        return;
                                    }

                                    // Small delay
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;

                                    // Paste
                                    execute_paste(app_clone.clone());

                                    update_tray_status(&app_clone, "success");
                                    let _ = app_clone.emit("status-changed", "success");
                                    std::thread::sleep(std::time::Duration::from_secs(2));
                                    update_tray_status(&app_clone, "idle");
                                    let _ = app_clone.emit("status-changed", "idle");
                                }
                            }
                            Err(e) => {
                                eprintln!("Transcription error: {}", e);
                                update_tray_status(&app_clone, "error");
                                let _ = app_clone.emit("status-changed", format!("error:{}", e));
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                update_tray_status(&app_clone, "idle");
                                let _ = app_clone.emit("status-changed", "idle");
                            }
                        }
                    });
                }
                None => break,
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel::<AudioCommand>(10);

    tauri::Builder::default()
        .manage(AppState {
            current_shortcut: Mutex::new(None),
            audio_sender: Mutex::new(Some(tx)),
            api_key: Mutex::new(String::new()),
            model: Mutex::new(String::from("gemini-3-pro-preview")),
            tray_icon: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            println!("App setup starting...");

            #[cfg(target_os = "macos")]
            {
                if !check_accessibility_permission() {
                    println!("Requesting accessibility permission...");
                }
            }

            // Start audio processing thread
            start_audio_processing(app.handle().clone(), rx);

            // Create tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &quit])?;

            // Build tray icon
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("AquaVoice - Ready")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.center();
                            let _ = window.set_decorations(true);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Store tray icon in app state
            let state = app.state::<AppState>();
            *state.tray_icon.lock().unwrap() = Some(tray);

            // Prevent window close from exiting the app
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(|event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            execute_paste,
            set_api_key,
            set_model,
            register_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
