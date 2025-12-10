use tauri::{AppHandle, Manager, Emitter};
use std::env;
use std::path::Path;
use tauri::path::BaseDirectory;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_dummy_game(
    handle: AppHandle,
    path: &str, // Relative path inside 'games' folder
    executable_name: &str,
    app_id: String,
) -> Result<String, String> {
    // Get the executable directory to look for config file
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let normalized_path = Path::new(path).to_string_lossy().to_string();

    let game_folder_path = exe_dir
        .join("games")
        .join(&app_id)
        .join(normalized_path);

    println!("Creating game folder: {:?}", game_folder_path);

    if let Err(e) = std::fs::create_dir_all(&game_folder_path) {
        return Err(format!("Failed to create game folder: {}", e));
    }

    // Resolve the runner executable from resources
    let resource_path = handle
        .path()
        .resolve("resources/runner.exe", BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;

    if !resource_path.exists() {
         return Err(format!("Runner executable not found at: {:?}", resource_path));
    }

    let target_executable_path = game_folder_path.join(executable_name);
    
    // Ensure parent directory for the executable exists (handling cases like 'win64/game.exe')
    if let Some(parent) = target_executable_path.parent() {
        if !parent.exists() {
            println!("Creating executable parent directory: {:?}", parent);
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("Failed to create executable parent directory: {}", e));
            }
        }
    }

    println!("Copying runner from {:?} to {:?}", resource_path, target_executable_path);

    match std::fs::copy(&resource_path, &target_executable_path) {
        Ok(_) => Ok(format!(
            "Dummy executable copied to: {:?}",
            target_executable_path
        )),
        Err(e) => Err(format!("Failed to copy dummy executable: {}", e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn start_game_process(
    handle: AppHandle,
    name: &str,
    path: &str,
    executable_name: &str,
    app_id: String,
    icon_url: Option<String>,
) -> Result<String, String> {
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let normalized_path = Path::new(path).to_string_lossy().to_string();

    let game_folder_path = exe_dir
        .join("games")
        .join(&app_id)
        .join(normalized_path);
        
    let executable_path = game_folder_path.join(executable_name);

    if !executable_path.exists() {
        return Err(format!("Executable not found at {:?}", executable_path));
    }

    println!("Starting process: {:?}", executable_path);

    let mut args = vec!["--title".to_string(), name.to_string()];
    if let Some(url) = icon_url {
        args.push("--icon".to_string());
        args.push(url);
    }

    let mut child = std::process::Command::new(&executable_path)
        .args(&args)
        .current_dir(game_folder_path)
        .spawn()
        .map_err(|e| format!("Failed to start process: {}", e))?;

    let pid = child.id();
    let app_handle = handle.clone();

    // Spawn a monitoring task
    tauri::async_runtime::spawn(async move {
        // Simple wait (blocking the thread, but it's a spawned thread so it's okay for now, 
        // ideally we'd use async process wait but std::process is sync. 
        // Tauri's spawn sends it to a thread pool.)
        // check status periodically or use wait() which blocks this thread.
        match child.wait() {
            Ok(status) => {
                println!("Process {} exited with status: {}", pid, status);
                let _ = app_handle.emit("game_exited", ());
            }
            Err(e) => {
                eprintln!("Failed to wait on process {}: {}", pid, e);
            }
        }
    });

    Ok("Process started successfully".to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_process(exec_name: String) -> Result<(), String> {
    let output = std::process::Command::new("taskkill")
        .arg("/F")
        .arg("/IM")
        .arg(exec_name)
        .output()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to stop process: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn fetch_game_list() -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut games: Vec<serde_json::Value> = Vec::new();

    // 1. Fetch from Discord API
    let discord_url = "https://discord.com/api/applications/detectable";
    if let Ok(res) = client.get(discord_url).send().await {
        if let Ok(list) = res.json::<Vec<serde_json::Value>>().await {
            games.extend(list);
        }
    }

    // 2. Fetch from Extended Gist
    let gist_url = "https://gist.githubusercontent.com/DeadSix27/b8e377c9fed6d98bff22dcdf8807e207/raw/52d1f2d31be7168a0486a3a355e06a2d751bdc44/gameslist.json";
    if let Ok(res) = client.get(gist_url).send().await {
        if let Ok(list) = res.json::<Vec<serde_json::Value>>().await {
            games.extend(list);
        }
    }

    // 3. Deduplicate by ID
    let mut unique_games = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for mut game in games {
        // Normalize 'icon_hash' to 'icon' for consistency across sources
        if game.get("icon").is_none() {
            if let Some(hash) = game.get("icon_hash").and_then(|h| h.as_str()) {
                game["icon"] = serde_json::Value::String(hash.to_string());
            }
        }

        if let Some(id) = game.get("id").and_then(|v| v.as_str()) {
            if seen_ids.insert(id.to_string()) {
                unique_games.push(game);
            }
        }
    }

    serde_json::to_string(&unique_games).map_err(|e| e.to_string())
}
