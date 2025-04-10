use crate::slint_ui::*;


pub fn page_one(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let ui_weak = ui_weak.clone();
    let page_one_logic = ui.global::<PageOneLogic>();

    page_one_logic.on_to_page_two(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_page(Pages::PageTwo);
        }
    });
}