use eframe::egui;
use std::sync::mpsc::SyncSender;

#[derive(Clone, Default)]
pub struct CollatzState {
    pub current_num: u64,
    pub current_len: u64,
    pub best_num: u64,
    pub best_len: u64,
    pub history: Vec<u64>, 
}

pub fn solve(tx: SyncSender<CollatzState>) {
    let mut state = CollatzState::default();
    
    // Iterate 1 to 1,000,000
    for i in 1..1_000_000 {
        let mut n = i;
        let mut len = 1;
        
        // Calculate Collatz Length
        while n > 1 {
            if n % 2 == 0 { 
                n /= 2; 
            } else { 
                n = 3 * n + 1; 
            }
            len += 1;
        }

        // Update State
        state.current_num = i;
        state.current_len = len;
        
        // Add to graph history (keep only last 400 points)
        state.history.push(len);
        if state.history.len() > 400 { 
            state.history.remove(0); 
        }

        // Check for new record
        if len > state.best_len {
            state.best_len = len;
            state.best_num = i;
        }

        if tx.send(state.clone()).is_err() { break; }
    }
}

pub fn render(state: &CollatzState, buffer: &mut Vec<u8>) {
    // Clear to black
    buffer.fill(0);

    // Draw the "History Graph"
    // Each pixel column represents one number checked
    let h = 300;
    let w = 400;
    
    for (x, &len) in state.history.iter().enumerate() {
        if x >= w { break; }
        
        // Scale height: Max known collatz length is ~525 for <1M
        // We scale so 525 fills the screen height (300px)
        let bar_height = ((len as f32 / 525.0) * h as f32) as usize;
        
        // Draw vertical line
        for y in 0..bar_height.min(h) {
            // Flip Y so 0 is at bottom
            let pixel_y = h - 1 - y;
            let idx = (pixel_y * w + x) * 3;
            
            if idx + 2 < buffer.len() {
                // Color gradient based on height (Blue -> Cyan -> White)
                let intensity = (y as u8).saturating_mul(2);
                buffer[idx] = 0;                        // R
                buffer[idx+1] = intensity;              // G
                buffer[idx+2] = 255 - (intensity / 2);  // B
            }
        }
    }
}

pub fn ui(state: &CollatzState, ui: &mut egui::Ui) {
    ui.heading("Problem 14: Collatz");
    
    ui.label(format!("Checking: {}", state.current_num));
    ui.label(format!("Length: {}", state.current_len));
    
    ui.separator();
    
    ui.heading("Current Record");
    ui.label(format!("Number: {}", state.best_num));
    ui.colored_label(egui::Color32::GREEN, format!("Length: {}", state.best_len));
}
