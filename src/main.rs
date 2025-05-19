// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

mod pages{
    pub mod home_page;
    pub mod schindler_page;
}

mod helpers{
}

// Import the centralized Slint module
mod slint_ui;
use slint_ui::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    slint_ui::HeaderLogic(&ui, ui_weak.clone());

    ui.global::<Pallet>().set_color_scheme(ColorScheme::Dark);

    pages::home_page::home_page(&ui, ui_weak.clone());
    pages::schindler_page::schindler_page(&ui, ui_weak.clone()).await;

    ui.run()?;

    Ok(())
}
