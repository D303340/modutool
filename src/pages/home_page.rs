use slint::Model;

use crate::slint_ui::*;


pub fn to_kollmorgen(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak = ui_weak.clone();
    if let Some(ui) = ui_weak.upgrade() {
        if let Err(e) = open::that("https://192.168.100.1") {
            eprintln!("Failed to open URL: {}", e);
        }
    }
}


pub fn home_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let ui_weak = ui_weak.clone();

    let ui_weak_schindler = ui_weak.clone();
    ui.global::<HomePageLogic>().on_to_schindler(move || {    
        if let Some(ui) = ui_weak_schindler.upgrade() {
            ui.set_current_page(Pages::SchindlerPage);
        }
    });

    let ui_weak_kollmorgen = ui_weak.clone();
    ui.global::<HomePageLogic>().on_to_kollmorgen(move || {
        if let Some(ui) = ui_weak_kollmorgen.upgrade() {
            to_kollmorgen(&ui, ui_weak_kollmorgen.clone());
            ui.window().set_minimized(true);
            ui.window().set_maximized(false);
        }
    });

}