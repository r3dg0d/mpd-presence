use lofty::{TaggedFileExt, PictureType, MimeType};
use std::path::Path;
use reqwest::blocking::Client;
//...
use reqwest::blocking::multipart;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref URL_CACHE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn get_album_art_url(path: &str) -> Option<String> {
    let path = Path::new(path);
    if !path.exists() {
        return None;
    }

    if let Ok(cache) = URL_CACHE.lock() {
        if let Some(url) = cache.get(path.to_str().unwrap_or_default()) {
            return Some(url.clone());
        }
    }

    // Use read_from_path from lofty specific to 0.15? 
    // Usually lofty::read_from_path works.
    let tagged_file = lofty::read_from_path(path).ok()?;
    let tag = tagged_file.primary_tag()?;

    // Pictures are in tag.pictures()
    let pictures = tag.pictures();
    
    // Explicitly finding cover front
    let picture = pictures.iter().find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first())?;

    let data = picture.data();
    let mime = picture.mime_type();
    
    let mime_str = match mime {
        MimeType::Png => "image/png",
        MimeType::Jpeg => "image/jpeg",
        MimeType::Tiff => "image/tiff",
        MimeType::Bmp => "image/bmp",
        MimeType::Gif => "image/gif",
        _ => "application/octet-stream",
    };

    let url = upload_to_0x0st(data, mime_str)?;
    
    if let Ok(mut cache) = URL_CACHE.lock() {
        cache.insert(path.to_str().unwrap_or_default().to_string(), url.clone());
    }

    Some(url)
}

fn upload_to_0x0st(data: &[u8], mime: &str) -> Option<String> {
    let client = Client::new();
    
    // We must provide a filename and mime for 0x0.st to process it correctly
    // Mime is crucial.
    let part = multipart::Part::bytes(data.to_vec())
        .file_name("art.jpg") 
        .mime_str(mime).ok()?;

    let form = multipart::Form::new()
        .part("file", part);

    let res = client.post("https://0x0.st")
        .multipart(form)
        .send().ok()?;

    if res.status().is_success() {
        let body = res.text().ok()?;
        // 0x0.st returns the URL in body, trimmed.
        let url = body.trim().to_string();
        // Check if it looks like a URL
        if url.starts_with("http") {
             Some(url)
        } else {
             None
        }
    } else {
        None
    }
}
