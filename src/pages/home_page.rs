use crate::slint_ui::*;


pub fn home_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let _home_page_logic = ui.global::<HomePageLogic>();

    let _ui_weak = ui_weak.clone();
}