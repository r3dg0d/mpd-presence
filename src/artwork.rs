use lofty::file::TaggedFileExt;
use lofty::picture::{PictureType, MimeType};
use std::path::Path;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::Deserialize;

lazy_static::lazy_static! {
    static ref URL_CACHE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesResult>,
}

#[derive(Deserialize)]
struct ItunesResult {
    #[serde(rename = "artworkUrl100")]
    artwork_url100: String,
}

pub fn get_album_art_url(path: &str, artist: &str, album: &str) -> Option<String> {
    let path_obj = Path::new(path);
    let cache_key = path.to_string();

    // Check cache
    if let Ok(cache) = URL_CACHE.lock() {
        if let Some(url) = cache.get(&cache_key) {
            return Some(url.clone());
        }
    }

    // Try local file first
    if let Some(url) = extract_and_upload_local(path_obj) {
        if let Ok(mut cache) = URL_CACHE.lock() {
            cache.insert(cache_key, url.clone());
        }
        return Some(url);
    }

    println!("Local art failed. Searching web for: {} - {}", artist, album);

    // Fallback to Web Search (iTunes API)
    if let Some(url) = search_itunes(artist, album) {
        if let Ok(mut cache) = URL_CACHE.lock() {
            cache.insert(cache_key, url.clone());
        }
        return Some(url);
    }

    None
}

fn extract_and_upload_local(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }

    println!("Attempting to read art from: {:?}", path);
    // read_from_path reads file and parses it
    let tagged_file = lofty::read_from_path(path).ok()?;
    let tag = tagged_file.primary_tag()?;

    let pictures = tag.pictures();
    if pictures.is_empty() {
        return None;
    }

    let picture = pictures.iter()
        .find(|p: &&lofty::picture::Picture| p.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first())?;

    let data = picture.data();
    let mime_enum = picture.mime_type();
    
    let mime_str = match mime_enum {
        Some(MimeType::Png) => "image/png",
        Some(MimeType::Jpeg) => "image/jpeg",
        Some(MimeType::Tiff) => "image/tiff",
        Some(MimeType::Bmp) => "image/bmp",
        Some(MimeType::Gif) => "image/gif",
        _ => "application/octet-stream",
    };

    println!("Uploading local art ({} bytes)...", data.len());
    upload_to_litterbox(data, mime_str)
}

fn search_itunes(artist: &str, album: &str) -> Option<String> {
    let client = Client::new();
    let term = format!("{} {}", artist, album);
    let url = "https://itunes.apple.com/search";

    let res = client.get(url)
        .query(&[
            ("term", term.as_str()),
            ("entity", "album"),
            ("limit", "1")
        ])
        .send().ok()?;

    if !res.status().is_success() {
        return None;
    }

    let data: ItunesResponse = res.json().ok()?;
    
    if let Some(result) = data.results.first() {
        // Upgrade quality 100x100 -> 600x600
        let hq_url = result.artwork_url100.replace("100x100bb", "600x600bb");
        println!("Found web art: {}", hq_url);
        Some(hq_url)
    } else {
        println!("No results found on web.");
        None
    }
}

fn upload_to_litterbox(data: &[u8], mime: &str) -> Option<String> {
    let client = Client::new();
    
    let part = multipart::Part::bytes(data.to_vec())
        .file_name("art.jpg") 
        .mime_str(mime).ok()?;

    let form = multipart::Form::new()
        .text("reqtype", "fileupload")
        .text("time", "12h") 
        .part("fileToUpload", part);

    let res = client.post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(form)
        .send().ok()?;

    if res.status().is_success() {
        let body = res.text().ok()?;
        let url = body.trim().to_string();
        if url.starts_with("http") {
             Some(url)
        } else {
             None
        }
    } else {
        None
    }
}
