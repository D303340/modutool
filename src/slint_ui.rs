slint::include_modules!();


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

pub fn HeaderLogic(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let Header = ui.global::<HeaderLogic>();
    let ui_weak = ui_weak.clone();
    
    Header.on_to_home_page(move || {
        if let Some(ui) = ui_weak.clone().upgrade() {
            to_home_page(&ui, ui_weak.clone());
        }
    });

    Header.set_maximized(false);
}