slint::include_modules!();

use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use slint::SharedString;


pub fn to_home_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak = ui_weak.clone();
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_current_page(Pages::Home);
    }
}

pub fn to_schindler_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak = ui_weak.clone();
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_current_page(Pages::SchindlerPage);
    }
}

pub fn to_settings_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak = ui_weak.clone();
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_current_page(Pages::SettingsPage);
    }
}

pub fn HeaderLogic(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let Header = ui.global::<HeaderLogic>();
    
    let ui_weak_home = ui_weak.clone();
    Header.on_to_home_page(move || {
        if let Some(ui) = ui_weak_home.upgrade() {
            to_home_page(&ui, ui_weak_home.clone());
        }
    });

    let ui_weak_settings = ui_weak.clone();
    Header.on_to_settings_page(move || {
        if let Some(ui) = ui_weak_settings.upgrade() {
            to_settings_page(&ui, ui_weak_settings.clone());
        }
    });

    Header.set_maximized(false);
}


#[derive(Serialize, Deserialize, Debug)]

struct TextStyle {
    font_family: String,
    font_size: u8,
    font_weight: u16,
    font_color: String,
    font_italic: bool
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    text: HashMap<String, TextStyle>,
}


pub fn SettingsPageLogic(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let SettingsPage = ui.global::<SettingsPageLogic>();
    
    let ui_weak_settings = ui_weak.clone();
    SettingsPage.on_save_settings(move || {
        if let Some(ui) = ui_weak_settings.upgrade() {
            let console_time_size = ui.global::<SettingsPageLogic>().get_console_time_size();
            let console_main_size = ui.global::<SettingsPageLogic>().get_console_main_size();


            // Create a HashMap to hold the text styles
            let mut text_styles: HashMap<String, TextStyle> = HashMap::new();

            // Insert the text styles into the HashMap
            text_styles.insert("consoleTimeText".to_string(), TextStyle {
                font_family: "Roboto".to_string(),
                font_size: console_time_size as u8,
                font_weight: 400 as u16,
                font_color: "#ffffff".to_string(),
                font_italic: false
            });

            text_styles.insert("consoleMainText".to_string(), TextStyle {
                font_family: "Roboto".to_string(),
                font_size: console_main_size as u8,
                font_weight: 400 as u16,
                font_color: "#ffffff".to_string(),
                font_italic: false
            });

            // Create the Settings struct with the text styles
            // You can add more fields to the Settings struct as needed
            let settings = Settings {
                text: text_styles
            };

            // Serialize the settings to a JSON string
            let  json_str = serde_json::to_string_pretty(&settings).unwrap();

            // Write the JSON string to a file
            // Normaly, this is put inside home/.config/modu-tool 
            // For debugging purposes, the file is stored in the project root directory
            if let Err(e) = fs::write("settings.json", json_str.clone()) {
                eprintln!("Failed to save settings: {}", e);
            } else {
                println!("\n\n{}\n\n", json_str);
                println!("Settings saved!");
            }

            load_pallet(&ui, ui_weak_settings.clone());
            println!("Pallet loaded!");

            // to_home_page(&ui, ui_weak_settings.clone());
        }
    });
}


pub fn load_pallet(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let Fonts = ui.global::<Fonts>();

    // Read the settings from the JSON file
    let contents = match fs::read_to_string("settings.json") {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read settings file: {}", e);
            return;
        }
    };
    
    // Deserialize the JSON string into the Settings struct
    let settings: Settings = match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse settings: {}", e);
            return;
        }
    };

    // (3) Use `config` however you need
    if let Some(console_style) = settings.text.get("consoleTimeText") {
        Fonts.set_console_time_text(
            font_style { 
                font_color: (ui.global::<Pallet>().get_console_time_text()), 
                font_family: (SharedString::from(console_style.font_family.clone())), 
                font_italic: (console_style.font_italic), 
                font_size: (console_style.font_size as f32), 
                font_weight: (console_style.font_weight as i32) 
            }
        );
    }
    if let Some(console_style) = settings.text.get("consoleMainText") {
        Fonts.set_console_main_text(
            font_style { 
                font_color: (ui.global::<Pallet>().get_console_main_text()), 
                font_family: (SharedString::from(console_style.font_family.clone())), 
                font_italic: (console_style.font_italic), 
                font_size: (console_style.font_size as f32), 
                font_weight: (console_style.font_weight as i32) 
            }
        );
    }


    println!("Deserialized settings: \n{:#?}", settings);
}