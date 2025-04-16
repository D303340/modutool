use crate::slint_ui::*;


pub fn home_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let home_page_logic = ui.global::<HomePageLogic>();
    let ui_weak = ui_weak.clone();

    home_page_logic.on_to_schindler(move || {    
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_page(Pages::SchindlerPage);
        }
    });

}