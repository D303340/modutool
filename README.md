# Slint & Rust Way Better Template
A way better template for slint rust.
## Making a new page

1. Create a new slint file in the `ui/views/pages`.

2. Past this template code in the new file:
    ```qt
    import { Page } from "../../components/page.slint";
    import { Pallet } from "../../components/style.slint";


    export global YourPageLogic {
        // Every property and/or callback you want to 
        // expose to Rust must be declared here...
    }

    export component YourPage inherits Page {
        // your code here...
    }
    ```
3. Inside `ui/components/pages.slint` add your page. for example:
    ```python
    import { AboutPage, AboutPageLogic } from "../views/pages/AboutPage.slint";
    export { AboutPage, AboutPageLogic }
    ``` 
    <sup>Note: export the page and logic, so it can be used in the app-window.slint</sup><br><br>


4. Inside `ui/app-window.slint` import your page and logic from 'ui/components/pages.slint'. for example:
    ```qt
    import { YourPage, YourPageLogic } from "./components/pages.slint";
    export { YourPage, YourPageLogic }

    // This is so Rust has access to global components
    // Export every imported component you want access inside Rust
    export { ColorScheme, Pallet }
    export { HomePageLogic }
    ```
5. in `src/pages` make a new rust file for your page.

6. Inside your rust file, add this template code. try keeping the function name the same as the page name. for example:
    ```rust
    use crate::slint_ui::*;

    pub fn your_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
        let ui_weak = ui_weak.clone();
    }
    ```
7. Inside `main.rs` add your page logic:
    ```rust
    mod pages{
        pub mod home_page;
    }
    ``` 
    <sup>Note: make sure the module is public.</sup><br><br>
