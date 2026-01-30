use mpd::{Client, Idle, Subsystem};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub file_path: Option<String>,
}

pub struct MpdHandler {
    conn: Client,
    music_dir: PathBuf,
}

impl MpdHandler {
    pub fn new(host: &str, music_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let conn = Client::connect(host)?;
        Ok(Self { conn, music_dir })
    }

    pub fn get_current_song(&mut self) -> Result<Option<SongInfo>, Box<dyn Error>> {
        if let Some(song) = self.conn.currentsong()? {
            let album = song.tags.iter()
                .find(|(k, _)| k == "Album")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "Unknown Album".to_string());
                
            Ok(Some(SongInfo {
                title: song.title.unwrap_or_else(|| "Unknown Title".to_string()),
                artist: song.artist.unwrap_or_else(|| "Unknown Artist".to_string()),
                album,
                file_path: Some(song.file),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn wait_for_change(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn.wait(&[Subsystem::Player])?; 
        Ok(())
    }
//...
    
    // Helper to resolve absolute path
    pub fn get_absolute_path(&self, relative_path: &str) -> PathBuf {
        self.music_dir.join(relative_path)
    }
}
