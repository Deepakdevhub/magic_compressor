use wasm_bindgen::prelude::*;
use std::io::Cursor;
use image::ImageOutputFormat;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// THIS IS THE HIGH-PERFORMANCE FUNCTION
#[wasm_bindgen]
pub fn compress_exact(image_data: &[u8], target_kb: usize) -> Vec<u8> {
    // 1. Load image (supports PNG/JPG)
    let img = match image::load_from_memory(image_data) {
        Ok(i) => i,
        Err(_) => return Vec::new(), // Return empty if error
    };

    // 2. Binary Search for Quality (The "Secret Sauce")
    let mut min_q = 5;
    let mut max_q = 100;
    let mut best_data: Vec<u8> = Vec::new();
    
    while min_q <= max_q {
        let mid_q = (min_q + max_q) / 2;
        
        let mut buffer = Cursor::new(Vec::new());
        // Try compressing at "mid" quality
        img.write_to(&mut buffer, ImageOutputFormat::Jpeg(mid_q as u8))
           .unwrap_or_default();
        
        let compressed = buffer.into_inner();
        let size_kb = compressed.len() / 1024;

        if size_kb <= target_kb {
            // It fits! Save it and try for higher quality
            best_data = compressed; 
            min_q = mid_q + 1; 
        } else {
            // Too big! Lower quality
            if mid_q == 0 { break; }
            max_q = mid_q - 1;
        }
    }

    // Return the best result
    if best_data.is_empty() {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Jpeg(5)).unwrap();
        return buffer.into_inner();
    }

    best_data
}