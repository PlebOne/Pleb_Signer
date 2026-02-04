//! Simple UI for Pleb Signer using iced

use std::sync::Arc;
use tokio::sync::Mutex;

use iced::{
    Element, Length, Task, Theme,
    widget::{button, column, container, row, text, scrollable, horizontal_space, text_input, checkbox},
};

use crate::keys::{KeyManager, KeyMetadata};
use crate::config::Config;
use crate::client::PlebSignerClient;
use crate::error::SignerError;

/// Main view states
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewState {
    #[default]
    Main,
    KeyManagement,
    Settings,
    AddKey,
    Bunker,
}

/// UI Messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(ViewState),
    
    // Key Management
    GenerateKey,
    KeyNameInput(String),
    ImportKeyInput(String),
    ImportKey,
    DeleteKey(String),
    SelectKey(String),
    KeyOperationComplete(Result<String, String>),
    RefreshKeys,
    KeysRefreshed(Vec<KeyMetadata>),
    
    // Settings
    ToggleAutoStart(bool),
    ToggleNotifications(bool),
    SaveSettings,
    SettingsSaved(Result<(), String>),
    
    // Bunker
    ToggleBunker(bool),
    GenerateBunkerUri,
    BunkerUriGenerated(Result<String, String>),
    CopyBunkerUri,
    
    // General
    Lock,
    Noop,
}

/// Main UI state
pub struct PlebSignerUi {
    view: ViewState,
    error_message: Option<String>,
    success_message: Option<String>,
    
    // Key management
    key_name_input: String,
    import_key_input: String,
    keys_list: Vec<KeyMetadata>,
    
    // Settings
    auto_start: bool,
    notifications_enabled: bool,
    
    // Bunker
    bunker_enabled: bool,
    bunker_uri: Option<String>,
    
    // Shared state
    key_manager: Arc<Mutex<KeyManager>>,
    config: Config,
}

impl Default for PlebSignerUi {
    fn default() -> Self {
        Self {
            view: ViewState::Main,
            error_message: None,
            success_message: None,
            key_name_input: String::new(),
            import_key_input: String::new(),
            keys_list: Vec::new(),
            auto_start: false,
            notifications_enabled: true,
            bunker_enabled: false,
            bunker_uri: None,
            key_manager: Arc::new(Mutex::new(KeyManager::new())),
            config: Config::default_config(),
        }
    }
}

impl PlebSignerUi {
    pub fn new(key_manager: Arc<Mutex<KeyManager>>, config: Config) -> (Self, Task<Message>) {
        let ui = Self {
            view: ViewState::Main,
            error_message: None,
            success_message: None,
            key_name_input: String::new(),
            import_key_input: String::new(),
            keys_list: Vec::new(),
            auto_start: config.general.auto_start,
            notifications_enabled: config.general.show_notifications,
            bunker_enabled: false,
            bunker_uri: None,
            key_manager,
            config,
        };
        
        // Load keys on startup
        let km = ui.key_manager.clone();
        let task = Task::perform(
            async move {
                let mut manager = km.lock().await;
                let _ = manager.load().await;
                manager.list_keys().into_iter().cloned().collect()
            },
            Message::KeysRefreshed,
        );
        
        (ui, task)
    }

    pub fn title(&self) -> String {
        "Pleb Signer".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(view) => {
                self.view = view;
                self.error_message = None;
                self.success_message = None;
                Task::none()
            }
            
            Message::KeyNameInput(name) => {
                self.key_name_input = name;
                Task::none()
            }
            
            Message::ImportKeyInput(key) => {
                self.import_key_input = key;
                Task::none()
            }
            
            Message::GenerateKey => {
                let name = self.key_name_input.clone();
                if name.is_empty() {
                    self.error_message = Some("Please enter a key name".into());
                    return Task::none();
                }
                
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let mut manager = km.lock().await;
                        match manager.generate_key(&name).await {
                            Ok(meta) => Ok(format!("Generated key: {}", meta.npub)),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::KeyOperationComplete,
                )
            }
            
            Message::ImportKey => {
                let name = self.key_name_input.clone();
                let secret = self.import_key_input.clone();
                
                if name.is_empty() {
                    self.error_message = Some("Please enter a key name".into());
                    return Task::none();
                }
                if secret.is_empty() {
                    self.error_message = Some("Please enter the private key".into());
                    return Task::none();
                }
                
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let mut manager = km.lock().await;
                        match manager.import_key(&name, &secret).await {
                            Ok(meta) => Ok(format!("Imported key: {}", meta.npub)),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::KeyOperationComplete,
                )
            }
            
            Message::DeleteKey(name) => {
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let mut manager = km.lock().await;
                        match manager.delete_key(&name).await {
                            Ok(_) => Ok("Key deleted".to_string()),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::KeyOperationComplete,
                )
            }
            
            Message::SelectKey(name) => {
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let mut manager = km.lock().await;
                        match manager.set_active_key(&name).await {
                            Ok(_) => Ok(format!("Active key: {}", name)),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::KeyOperationComplete,
                )
            }
            
            Message::KeyOperationComplete(result) => {
                match result {
                    Ok(msg) => {
                        self.success_message = Some(msg);
                        self.error_message = None;
                        self.key_name_input.clear();
                        self.import_key_input.clear();
                        self.view = ViewState::KeyManagement;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        self.success_message = None;
                    }
                }
                
                // Refresh keys list
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let manager = km.lock().await;
                        manager.list_keys().into_iter().cloned().collect()
                    },
                    Message::KeysRefreshed,
                )
            }
            
            Message::RefreshKeys => {
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let manager = km.lock().await;
                        manager.list_keys().into_iter().cloned().collect()
                    },
                    Message::KeysRefreshed,
                )
            }
            
            Message::KeysRefreshed(keys) => {
                self.keys_list = keys;
                Task::none()
            }
            
            Message::ToggleAutoStart(v) => {
                self.auto_start = v;
                Task::none()
            }
            
            Message::ToggleNotifications(v) => {
                self.notifications_enabled = v;
                Task::none()
            }
            
            Message::SaveSettings => {
                let mut config = self.config.clone();
                config.general.auto_start = self.auto_start;
                config.general.show_notifications = self.notifications_enabled;
                
                Task::perform(
                    async move {
                        config.save().await.map_err(|e| e.to_string())
                    },
                    Message::SettingsSaved,
                )
            }
            
            Message::SettingsSaved(result) => {
                match result {
                    Ok(()) => {
                        self.success_message = Some("Settings saved".into());
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        self.success_message = None;
                    }
                }
                Task::none()
            }
            
            Message::Lock => {
                // Lock the key manager
                let km = self.key_manager.clone();
                Task::perform(
                    async move {
                        let mut manager = km.lock().await;
                        manager.lock();
                        Ok::<(), String>(())
                    },
                    |_: Result<(), String>| Message::Noop,
                )
            }
            
            Message::ToggleBunker(enabled) => {
                self.bunker_enabled = enabled;
                if enabled {
                    // Call D-Bus to start the bunker
                    Task::perform(
                        async move {
                            match PlebSignerClient::new("pleb-signer-ui").await {
                                Ok(client) => {
                                    client.start_bunker().await
                                        .map_err(|e| e.to_string())
                                }
                                Err(e) => Err(e.to_string())
                            }
                        },
                        Message::BunkerUriGenerated,
                    )
                } else {
                    // Call D-Bus to stop the bunker
                    self.bunker_uri = None;
                    Task::perform(
                        async move {
                            if let Ok(client) = PlebSignerClient::new("pleb-signer-ui").await {
                                let _ = client.stop_bunker().await;
                            }
                            Ok::<(), String>(())
                        },
                        |_| Message::Noop,
                    )
                }
            }
            
            Message::GenerateBunkerUri => {
                // Call D-Bus to get or start the bunker
                Task::perform(
                    async move {
                        match PlebSignerClient::new("pleb-signer-ui").await {
                            Ok(client) => {
                                // First try to get existing URI, if not start bunker
                                match client.get_bunker_state().await {
                                    Ok(state) if state.contains("WaitingForConnection") || state.contains("Connected") => {
                                        client.get_bunker_uri().await.map_err(|e| e.to_string())
                                    }
                                    _ => {
                                        client.start_bunker().await.map_err(|e| e.to_string())
                                    }
                                }
                            }
                            Err(e) => Err(e.to_string())
                        }
                    },
                    Message::BunkerUriGenerated,
                )
            }
            
            Message::BunkerUriGenerated(result) => {
                match result {
                    Ok(uri) => {
                        self.bunker_uri = Some(uri);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        self.bunker_enabled = false;
                    }
                }
                Task::none()
            }
            
            Message::CopyBunkerUri => {
                if let Some(ref uri) = self.bunker_uri {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(uri.clone());
                        self.success_message = Some("Bunker URI copied to clipboard!".into());
                    }
                }
                Task::none()
            }
            
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> = match self.view {
            ViewState::Main => self.view_main(),
            ViewState::KeyManagement => self.view_keys(),
            ViewState::Settings => self.view_settings(),
            ViewState::AddKey => self.view_add_key(),
            ViewState::Bunker => self.view_bunker(),
        };
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
    
    fn view_main(&self) -> Element<Message> {
        let header = row![
            text("⚡ Pleb Signer").size(28),
            horizontal_space(),
            button(text("Keys")).on_press(Message::NavigateTo(ViewState::KeyManagement)),
            button(text("Bunker")).on_press(Message::NavigateTo(ViewState::Bunker)),
            button(text("Settings")).on_press(Message::NavigateTo(ViewState::Settings)),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);
        
        let active_key_text = if let Some(active) = self.keys_list.iter().find(|k| k.is_active) {
            format!("Active: {} ({}...)", active.name, &active.npub[..20.min(active.npub.len())])
        } else if self.keys_list.is_empty() {
            "No keys configured".to_string()
        } else {
            "No active key selected".to_string()
        };
        
        let bunker_status = if self.bunker_enabled {
            "🌐 Bunker: Active (remote signing enabled)"
        } else {
            "Bunker: Off"
        };
        
        let status = column![
            text("Status: Ready").size(16),
            text(active_key_text).size(14),
            text(format!("Keys: {}", self.keys_list.len())).size(14),
            text(bunker_status).size(14),
        ]
        .spacing(8);
        
        let mut content = column![header, status].spacing(30).padding(10);
        
        if let Some(ref msg) = self.success_message {
            content = content.push(
                text(msg).size(14).color(iced::Color::from_rgb(0.2, 0.8, 0.2))
            );
        }
        
        if let Some(ref err) = self.error_message {
            content = content.push(
                text(err).size(14).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
            );
        }
        
        content.into()
    }
    
    fn view_keys(&self) -> Element<Message> {
        let header = row![
            button(text("← Back")).on_press(Message::NavigateTo(ViewState::Main)),
            text("Key Management").size(24),
            horizontal_space(),
            button(text("+ Add Key")).on_press(Message::NavigateTo(ViewState::AddKey)),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);
        
        let keys_list: Element<Message> = if self.keys_list.is_empty() {
            container(
                column![
                    text("🔑").size(48),
                    text("No keys yet").size(16),
                    text("Click 'Add Key' to get started").size(14),
                ]
                .spacing(10)
                .align_x(iced::Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fixed(200.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let keys: Vec<Element<Message>> = self.keys_list
                .iter()
                .map(|key| {
                    let active_indicator = if key.is_active { "● " } else { "○ " };
                    let name = key.name.clone();
                    let name_for_select = key.name.clone();
                    let name_for_delete = key.name.clone();
                    
                    container(
                        row![
                            column![
                                text(format!("{}{}", active_indicator, name)).size(16),
                                text(format!("{}...", &key.npub[..30.min(key.npub.len())])).size(12),
                            ]
                            .spacing(4),
                            horizontal_space(),
                            if !key.is_active {
                                button(text("Set Active")).on_press(Message::SelectKey(name_for_select))
                            } else {
                                button(text("✓ Active")).style(button::success)
                            },
                            button(text("Delete")).on_press(Message::DeleteKey(name_for_delete)),
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(container::bordered_box)
                    .into()
                })
                .collect();
            
            scrollable(column(keys).spacing(10)).height(Length::Fill).into()
        };
        
        let mut content = column![header, keys_list].spacing(20);
        
        if let Some(ref msg) = self.success_message {
            content = content.push(
                text(msg).size(14).color(iced::Color::from_rgb(0.2, 0.8, 0.2))
            );
        }
        
        if let Some(ref err) = self.error_message {
            content = content.push(
                text(err).size(14).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
            );
        }
        
        content.into()
    }
    
    fn view_add_key(&self) -> Element<Message> {
        let header = row![
            button(text("← Back")).on_press(Message::NavigateTo(ViewState::KeyManagement)),
            text("Add Key").size(24),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);
        
        let name_input = column![
            text("Key Name").size(14),
            text_input("My Key", &self.key_name_input)
                .on_input(Message::KeyNameInput)
                .padding(10)
                .width(Length::Fixed(350.0)),
        ]
        .spacing(5);
        
        let generate_section = column![
            text("Generate New Key").size(16),
            button(text("Generate Random Key"))
                .on_press(Message::GenerateKey)
                .padding([10, 20]),
        ]
        .spacing(10);
        
        let import_section = column![
            text("Or Import Existing Key").size(16),
            text_input("nsec1... or hex private key", &self.import_key_input)
                .on_input(Message::ImportKeyInput)
                .padding(10)
                .width(Length::Fixed(350.0))
                .secure(true),
            button(text("Import Key"))
                .on_press(Message::ImportKey)
                .padding([10, 20]),
        ]
        .spacing(10);
        
        let mut content = column![
            header,
            name_input,
            generate_section,
            import_section,
        ]
        .spacing(25);
        
        if let Some(ref err) = self.error_message {
            content = content.push(
                text(err).size(14).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
            );
        }
        
        content.into()
    }
    
    fn view_settings(&self) -> Element<Message> {
        let header = row![
            button(text("← Back")).on_press(Message::NavigateTo(ViewState::Main)),
            text("Settings").size(24),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);
        
        let auto_start_checkbox = checkbox("Start on login", self.auto_start)
            .on_toggle(Message::ToggleAutoStart);
        
        let notifications_checkbox = checkbox("Show notifications", self.notifications_enabled)
            .on_toggle(Message::ToggleNotifications);
        
        let save_btn = button(text("Save Settings"))
            .on_press(Message::SaveSettings)
            .padding([10, 20]);
        
        let mut content = column![
            header,
            auto_start_checkbox,
            notifications_checkbox,
            save_btn,
        ]
        .spacing(20);
        
        if let Some(ref msg) = self.success_message {
            content = content.push(
                text(msg).size(14).color(iced::Color::from_rgb(0.2, 0.8, 0.2))
            );
        }
        
        if let Some(ref err) = self.error_message {
            content = content.push(
                text(err).size(14).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
            );
        }
        
        content.into()
    }
    
    fn view_bunker(&self) -> Element<Message> {
        let header = row![
            button(text("← Back")).on_press(Message::NavigateTo(ViewState::Main)),
            text("Bunker Mode (NIP-46)").size(24),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);
        
        let description = column![
            text("Remote Signing via Nostr Relays").size(16),
            text("").size(8),
            text("Bunker mode allows you to sign events from remote devices").size(14),
            text("(phones, other computers, web apps) without exposing your").size(14),
            text("private key. The signing requests travel through Nostr relays.").size(14),
        ]
        .spacing(2);
        
        let enable_toggle = checkbox("Enable Bunker Mode", self.bunker_enabled)
            .on_toggle(Message::ToggleBunker);
        
        let uri_section: Element<Message> = if self.bunker_enabled {
            if let Some(ref uri) = self.bunker_uri {
                let display_uri: String = if uri.len() > 60 {
                    format!("{}...", &uri[..60])
                } else {
                    uri.clone()
                };
                
                column![
                    text("Connection URI:").size(14),
                    container(
                        text(display_uri).size(12)
                    )
                    .padding(10)
                    .style(container::bordered_box)
                    .width(Length::Fill),
                    text("").size(4),
                    row![
                        button(text("📋 Copy URI")).on_press(Message::CopyBunkerUri),
                        button(text("🔄 Regenerate")).on_press(Message::GenerateBunkerUri),
                    ]
                    .spacing(10),
                    text("").size(12),
                    text("How to use:").size(14),
                    text("1. Copy the URI above").size(12),
                    text("2. In your remote Nostr client, look for 'Login with Bunker'").size(12),
                    text("   or 'NIP-46 / Nostr Connect' option").size(12),
                    text("3. Paste this URI or scan it as QR code").size(12),
                    text("4. Your signing requests will appear here").size(12),
                ]
                .spacing(4)
                .into()
            } else {
                column![
                    text("Generating connection URI...").size(14),
                ]
                .into()
            }
        } else {
            column![
                text("").size(8),
                text("Enable bunker mode to generate a connection URI").size(14),
            ]
            .into()
        };
        
        let mut content = column![
            header,
            description,
            text("").size(10),
            enable_toggle,
            text("").size(10),
            uri_section,
        ]
        .spacing(10);
        
        if let Some(ref msg) = self.success_message {
            content = content.push(
                text(msg).size(14).color(iced::Color::from_rgb(0.2, 0.8, 0.2))
            );
        }
        
        if let Some(ref err) = self.error_message {
            content = content.push(
                text(err).size(14).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
            );
        }
        
        content.into()
    }
    
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// Run the UI application
pub fn run_ui(
    key_manager: Arc<Mutex<KeyManager>>,
    config: Config,
) -> Result<(), SignerError> {
    iced::application("Pleb Signer", PlebSignerUi::update, PlebSignerUi::view)
        .theme(PlebSignerUi::theme)
        .window_size((550.0, 450.0))
        .run_with(move || PlebSignerUi::new(key_manager, config))
        .map_err(|e| SignerError::ConfigError(format!("UI error: {}", e)))?;
    
    Ok(())
}