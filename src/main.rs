mod mpd_handler;
mod discord_handler;
mod artwork;

use std::error::Error;
use std::path::PathBuf;
use std::env;
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_APP_ID: &str = "123456789012345678"; // Placeholder
const RETRY_DELAY: u64 = 5;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting MPD Presence...");

    // Determine Music Directory
    let music_dir = env::var("MPD_MUSIC_DIR")
        .map(PathBuf::from)
        .ok()
        .or_else(|| dirs::audio_dir())
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Music"));

    println!("Music Directory: {:?}", music_dir);

    // Get App ID
    let app_id = env::var("DISCORD_APP_ID").unwrap_or_else(|_| DEFAULT_APP_ID.to_string());
    if app_id == DEFAULT_APP_ID {
        println!("WARNING: Using placeholder Discord App ID. Set DISCORD_APP_ID env var.");
    }

    loop {
        if let Err(e) = run_loop(&app_id, music_dir.clone()) {
            println!("Error in main loop: {}. Retrying in {}s...", e, RETRY_DELAY);
            sleep(Duration::from_secs(RETRY_DELAY));
        }
    }
}

fn run_loop(app_id: &str, music_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut mpd = mpd_handler::MpdHandler::new("127.0.0.1:6600", music_dir)?;
    let mut discord = discord_handler::DiscordHandler::new(app_id)?;

    println!("Connected to MPD and Discord RPC.");

    loop {
        if let Some(song) = mpd.get_current_song()? {
            println!("Now Playing: {} - {}", song.artist, song.title);
            
            let art_url = if let Some(rel_path) = &song.file_path {
                let abs_path = mpd.get_absolute_path(rel_path);
                artwork::get_album_art_url(abs_path.to_str().unwrap_or_default())
            } else {
                None
            };

            discord.update_presence(
                &song.title,
                &song.artist,
                &song.album,
                art_url.as_deref(),
            )?;
        } else {
            println!("MPD stopped or paused.");
            // Clear presence or show "Idle"?
            // discord.clear_presence()?; // Optional
        }

        mpd.wait_for_change()?;
    }
}
