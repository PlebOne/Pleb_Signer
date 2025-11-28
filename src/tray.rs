//! System tray integration for Pleb Signer
//! 
//! Uses ksni which implements the StatusNotifierItem D-Bus protocol
//! (org.kde.StatusNotifierItem) supported by Cosmic, KDE, GNOME, etc.

use ksni::{Icon, Tray, TrayService};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Shared state between tray and main app
pub struct TrayState {
    pub is_locked: AtomicBool,
    pub quit_requested: AtomicBool,
    pub show_requested: AtomicBool,
    pub bunker_enabled: AtomicBool,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            is_locked: AtomicBool::new(false),
            quit_requested: AtomicBool::new(false),
            show_requested: AtomicBool::new(false),
            bunker_enabled: AtomicBool::new(false),
        }
    }
}

/// Generate a simple key icon as ARGB pixel data
/// This creates a 22x22 golden key icon
fn generate_key_icon() -> Vec<u8> {
    let size = 22;
    let mut pixels = vec![0u8; size * size * 4]; // ARGB format
    
    // Colors (ARGB format: Alpha, Red, Green, Blue)
    let gold: [u8; 4] = [255, 218, 165, 32];      // Golden color
    let dark_gold: [u8; 4] = [255, 184, 134, 11]; // Darker gold for outline
    let transparent: [u8; 4] = [0, 0, 0, 0];
    
    // Helper to set pixel
    let set_pixel = |pixels: &mut Vec<u8>, x: usize, y: usize, color: [u8; 4]| {
        if x < size && y < size {
            let idx = (y * size + x) * 4;
            pixels[idx] = color[0];     // A
            pixels[idx + 1] = color[1]; // R
            pixels[idx + 2] = color[2]; // G
            pixels[idx + 3] = color[3]; // B
        }
    };
    
    // Initialize with transparent
    for y in 0..size {
        for x in 0..size {
            set_pixel(&mut pixels, x, y, transparent);
        }
    }
    
    // Draw key head (circular part) - centered around (7, 7), radius ~4
    for y in 3..12 {
        for x in 3..12 {
            let dx = x as f32 - 7.0;
            let dy = y as f32 - 7.0;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= 4.5 && dist >= 2.0 {
                set_pixel(&mut pixels, x, y, gold);
            }
        }
    }
    
    // Draw key shaft (horizontal line from head to right)
    for x in 10..20 {
        set_pixel(&mut pixels, x, 7, gold);
        set_pixel(&mut pixels, x, 8, gold);
    }
    
    // Draw key teeth (small rectangles pointing down)
    // Tooth 1
    for y in 8..12 {
        set_pixel(&mut pixels, 14, y, gold);
        set_pixel(&mut pixels, 15, y, gold);
    }
    // Tooth 2
    for y in 8..11 {
        set_pixel(&mut pixels, 17, y, gold);
        set_pixel(&mut pixels, 18, y, gold);
    }
    
    // Add some outline/depth with darker gold
    set_pixel(&mut pixels, 10, 6, dark_gold);
    set_pixel(&mut pixels, 19, 6, dark_gold);
    set_pixel(&mut pixels, 19, 9, dark_gold);
    
    pixels
}

/// System tray icon implementation
pub struct PlebSignerTray {
    state: Arc<TrayState>,
    icon_pixels: Vec<u8>,
}

impl PlebSignerTray {
    pub fn new(state: Arc<TrayState>) -> Self {
        Self { 
            state,
            icon_pixels: generate_key_icon(),
        }
    }
}

impl Tray for PlebSignerTray {
    fn id(&self) -> String {
        "pleb-signer".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![Icon {
            width: 22,
            height: 22,
            data: self.icon_pixels.clone(),
        }]
    }

    fn title(&self) -> String {
        if self.state.is_locked.load(Ordering::Relaxed) {
            "Pleb Signer (Locked)".into()
        } else {
            "Pleb Signer".into()
        }
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::ApplicationStatus
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let is_locked = self.state.is_locked.load(Ordering::Relaxed);
        let bunker_enabled = self.state.bunker_enabled.load(Ordering::Relaxed);
        
        vec![
            StandardItem {
                label: format!("Status: {}", if is_locked { "🔒 Locked" } else { "🟢 Ready" }),
                enabled: false,
                ..Default::default()
            }.into(),
            StandardItem {
                label: format!("Bunker: {}", if bunker_enabled { "🌐 Active" } else { "⭘ Off" }),
                enabled: false,
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Show Window".into(),
                activate: Box::new(|this: &mut Self| {
                    this.state.show_requested.store(true, Ordering::Relaxed);
                    info!("Show window requested from tray");
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|this: &mut Self| {
                    this.state.quit_requested.store(true, Ordering::Relaxed);
                    info!("Quit requested from tray");
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        // Called when the tray icon is clicked
        self.state.show_requested.store(true, Ordering::Relaxed);
        info!("Tray icon clicked - show window requested");
    }
}

/// Start the system tray in a background thread
/// Returns the shared state that can be used to communicate with the tray
pub fn start_tray() -> Arc<TrayState> {
    let state = Arc::new(TrayState::new());
    let tray_state = Arc::clone(&state);

    std::thread::spawn(move || {
        info!("Starting system tray (StatusNotifierItem)...");
        let tray = PlebSignerTray::new(tray_state);
        let service = TrayService::new(tray);
        
        // This blocks the thread
        if let Err(e) = service.run() {
            tracing::error!("System tray error: {:?}", e);
        }
    });

    // Give the tray a moment to initialize
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    info!("System tray started");
    state
}
