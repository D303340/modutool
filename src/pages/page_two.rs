use crate::slint_ui::*;


pub fn page_two(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let ui_weak = ui_weak.clone();
    ui.global::<PageTwoLogic>().on_to_page_one(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_page(Pages::PageOne);
        }
    });
}