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

    println!("Attempting to read art from: {:?}", path);
    // Use read_from_path from lofty specific to 0.15? 
    let tagged_file = match lofty::read_from_path(path) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to read file: {}", e);
            return None;
        }
    };
    let tag = match tagged_file.primary_tag() {
        Some(t) => t,
        None => {
            println!("No primary tag found");
            return None;
        }
    };

    // Pictures are in tag.pictures()
    let pictures = tag.pictures();
    println!("Found {} pictures", pictures.len());
    
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
    println!("Uploading art ({} bytes, {})...", data.len(), mime_str);

    let url = upload_to_litterbox(data, mime_str)?;
    println!("Uploaded Art URL: {}", url);
    
    if let Ok(mut cache) = URL_CACHE.lock() {
        cache.insert(path.to_str().unwrap_or_default().to_string(), url.clone());
    }

    Some(url)
}

fn upload_to_litterbox(data: &[u8], mime: &str) -> Option<String> {
    let client = Client::new();
    
    // Litterbox expects 'reqtype=fileupload', 'time=1h', and 'fileToUpload'.
    // We'll use 12h just to be safe.
    let part = multipart::Part::bytes(data.to_vec())
        .file_name("art.jpg") 
        .mime_str(mime).ok()?;

    let form = multipart::Form::new()
        .text("reqtype", "fileupload")
        .text("time", "12h") 
        .part("fileToUpload", part);

    let res = match client.post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(form)
        .send() {
            Ok(r) => r,
            Err(e) => {
                println!("Upload failed: {}", e);
                return None;
            }
        };

    if res.status().is_success() {
        let body = res.text().ok()?;
        let url = body.trim().to_string();
        if url.starts_with("http") {
             Some(url)
        } else {
             println!("Invalid response from Litterbox: {}", url);
             None
        }
    } else {
        println!("Litterbox returned status: {}", res.status());
        None
    }
}
