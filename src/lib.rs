use wasm_bindgen::prelude::*;
use std::io::Cursor;
use image::{ImageOutputFormat, GenericImageView};

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// SECURITY CHECK (Keep your domain protection)
fn is_authorized_domain() -> bool {
    let window = web_sys::window().expect("no global `window` exists");
    let location = window.location();
    let hostname = location.hostname().unwrap_or_default();
    
    if hostname.contains("localhost") || 
       hostname.contains("127.0.0.1") || 
       hostname.contains("pages.dev") || 
       hostname.contains("netlify.app") {
        return true;
    }
    return false; // Change to true if you want to disable protection while testing
}

#[wasm_bindgen]
pub fn compress_exact(image_data: &[u8], target_kb: usize) -> Vec<u8> {
    if !is_authorized_domain() { return Vec::new(); }

    // 1. Load the image
    let mut img = match image::load_from_memory(image_data) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    // 2. PRE-CHECK: If the image is massive (e.g. 4000px from phone) and target is small (50kb),
    // resize it immediately to something reasonable (e.g. 1500px) to start.
    let (w, h) = img.dimensions();
    if target_kb < 100 && (w > 1500 || h > 1500) {
        img = img.resize(1500, 1500 * h / w, image::imageops::FilterType::Triangle);
    }

    let mut best_data: Vec<u8> = Vec::new();
    let mut attempt_resize_factor = 1.0;

    // 3. OUTER LOOP: Resize loop (Shrink dimensions if needed)
    // We try up to 3 times: Original Size -> 75% Size -> 50% Size
    for _ in 0..3 {
        // Apply resizing if this isn't the first attempt
        if attempt_resize_factor < 1.0 {
             let (new_w, new_h) = ((w as f32 * attempt_resize_factor) as u32, (h as f32 * attempt_resize_factor) as u32);
             img = img.resize(new_w, new_h, image::imageops::FilterType::Triangle);
        }

        // 4. INNER LOOP: Binary Search for Quality (The original logic)
        let mut min_q = 5;
        let mut max_q = 100;
        
        while min_q <= max_q {
            let mid_q = (min_q + max_q) / 2;
            let mut buffer = Cursor::new(Vec::new());
            
            // Try compressing
            img.write_to(&mut buffer, ImageOutputFormat::Jpeg(mid_q as u8)).unwrap_or_default();
            let compressed = buffer.into_inner();
            let size_kb = compressed.len() / 1024;

            if size_kb <= target_kb {
                best_data = compressed; // Found a valid one!
                min_q = mid_q + 1;      // Try to get better quality
            } else {
                if mid_q == 0 { break; }
                max_q = mid_q - 1;      // Too big, lower quality
            }
        }

        // If we found a valid file, STOP everything and return it!
        if !best_data.is_empty() {
            return best_data;
        }

        // If we didn't find a valid file, SHRINK the image and try the loop again
        attempt_resize_factor *= 0.75; 
    }

    // Fallback: If absolutely nothing worked, return the smallest attempt
    if best_data.is_empty() {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Jpeg(10)).unwrap();
        return buffer.into_inner();
    }

    best_data
}