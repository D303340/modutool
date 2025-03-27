// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
// use std::time::Duration;

use mqtt::{MqttMessage, MqttWorker};
use rumqttc::v5::{AsyncClient, Event, MqttOptions};
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::Transport;

use tokio::time::{sleep, Duration};

use rumqttc::LastWill;

use slint::invoke_from_event_loop;
use slint::SharedString;

mod helper;
mod comport;
mod mqtt;


slint::include_modules!();



fn navigation(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {

    // ============================================= //
    //                 ~~ Global ~~                  //
    // ============================================= //
    {
        let ui_weak = ui_weak.clone();
        ui.on_back_to_menu_page(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_page(Page::Menu);
            }
        });
    }



    // ============================================= //
    //                ~~ HOME MENU ~~                //
    // ============================================= //
    {
        // Modu-soft.kne Click
        {
            let ui_weak = ui_weak.clone();
            ui.on_modusoft_kne(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::ModuSoftKneMenu);
                }
            });
        }
        
        // Modu-soft.ots Click
        {
            let ui_weak = ui_weak.clone();
            ui.on_modusoft_ots(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::OtisMenu);
                }
            });
        }

        // Modu-soft.thy Click
        {
            let ui_weak = ui_weak.clone();
            ui.on_modusoft_thy(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::Menu);
                }
            });
        }

        // Modu-soft.ora Click
        {
            let ui_weak = ui_weak.clone();
            ui.on_modusoft_ora(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::Menu);
                }
            });
        }

        // Modu-soft.sch Click
        {
            let ui_weak = ui_weak.clone();
            ui.on_modusoft_sch(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::ModuSoftSchMenu);
                }
            });
        }
    }



    // ============================================= //
    //                ~~ OTIS MENU ~~                //
    // ============================================= //
    {
        // To Otis (Single) PAGE
        {
            let ui_weak = ui_weak.clone();
            ui.on_otis_single_page(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::OtisPage);
                }
            });
        }
        
        // To Otis (Double) PAGE
        {
            let ui_weak = ui_weak.clone();
            ui.on_otis_double_page(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_page(Page::OtisDoublePage);
                }
            });
        }
    }



    // ================================================== //
    //                ~~ SCHINDLER MENU ~~                //
    // ================================================== //
    {
        // To Schindler PAGE
        let ui_weak = ui_weak.clone();
        ui.on_schindler_page(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_page(Page::SchindlerTestPage);
            }
        });
    }



}


fn otis_page(ui: &AppWindow, 
            ui_weak: slint::Weak<AppWindow>,
            mqtt_worker: &MqttWorker) {
    // // On Esc button click
    // {
    //     let _ui_weak = ui_weak.clone();
    //     ui.on_click_esc(move || {
    //         std::thread::spawn(move || {
    //             // Create a Tokio runtime for the MQTT client.
    //             let rt = tokio::runtime::Runtime::new().unwrap();
    //             rt.block_on(async {
    //                 // Configure the temporary MQTT client (adjust broker as needed).
    //                 let mut mqttoptions = MqttOptions::new("temp_receiver", "172.23.128.1", 1883);
    //                 mqttoptions.set_keep_alive(Duration::from_secs(5));
    //                 let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
                    
    //                 // Subscribe to all topics (so retained messages are sent immediately).
    //                 client.subscribe("#", QoS::AtMostOnce).await.unwrap();
    //                 println!("Subscribed to all topics. Waiting for messages...");
        
    //                 // Define a deadline 5 seconds from now.
    //                 let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    //                 while tokio::time::Instant::now() < deadline {
    //                     // Poll for incoming MQTT events.
    //                     if let Ok(Event::Incoming(Packet::Publish(p))) = eventloop.poll().await {
    //                         // Convert topic and payload to strings.
    //                         let topic = String::from_utf8_lossy(&p.topic);
    //                         let payload = String::from_utf8_lossy(&p.payload);
    //                         // Print in the format: the/topic/from/mqtt: "value"
    //                         println!("{}: \"{}\"", topic, payload);
    //                     }
    //                 }
    //                 println!("Done receiving messages.");
    //             });
    //         });
    //     });
    // }

    // // On Up button click
    // {
    //     let _ui_weak = ui_weak.clone();
    //     let mqtt_channel = mqtt_worker.command_tx.clone();
    //     ui.on_click_down(move || {
    //         // Simply send a message over MQTT when the button is clicked.
    //         if let Err(err) = mqtt_channel.send(MqttMessage::Publish {
    //             topic: "otis/mqtt".to_string(),
    //             payload: "0".to_string(),
    //             retain: false,
    //         }) {
    //             eprintln!("Error sending MQTT message: {:?}", err);
    //         }
    //     });
    // }

    // // On Down button click
    // {
    //     let mqtt_channel = mqtt_worker.command_tx.clone();
    //     ui.on_click_down(move || {
    //         // Simply send a message over MQTT when the button is clicked.
    //         if let Err(err) = mqtt_channel.send(MqttMessage::Publish {
    //             topic: "otis/mqtt".to_string(),
    //             payload: "0".to_string(),
    //             retain: false,
    //         }) {
    //             eprintln!("Error sending MQTT message: {:?}", err);
    //         }
    //     });
    // }

    // // On Ok button click
    // {
        // let mqtt_channel = mqtt_worker.command_tx.clone();
    
        // ui.on_click_ok(move || {
        //     if let Some(_ui) = ui_weak.upgrade() {
        //         let ui_thread = ui_weak.clone();
        //         let mqtt_channel = mqtt_channel.clone();
        
        //         std::thread::spawn(move || {
        //             match comport::open_com_port() {
        //                 Ok(mut port) => {
        //                     if let Err(e) = comport::write_to_com_port(&mut *port, "r\n") {
        //                         eprintln!("Error writing to COM port: {}", e);
        //                         return;
        //                     }
        //                     match comport::continuous_read(&mut *port) {
        //                         Ok(response) => {
        //                             // Clone the response for the UI update,
        //                             // so the original can be used for MQTT.
        //                             let response_for_ui = response.clone();
        //                             let _ = invoke_from_event_loop(move || {
        //                                 if let Some(ui) = ui_thread.upgrade() {
        //                                     ui.set_lcd_text(helper::string_to_model_rc(response_for_ui));
        //                                 }
        //                             });
        //                             // Send the COM port response to MQTT.
        //                             if let Err(err) = mqtt_channel.send(MqttMessage::Publish {
        //                                 topic: String::from("otis/comport"),
        //                                 payload: String::from(format!("{}", response)),
        //                                 retain: false,
        //                             }) {
        //                                 eprintln!("Error sending MQTT message: {:?}", err);
        //                             }
        //                         }
        //                         Err(e) => {
        //                             eprintln!("Error reading from COM port: {}", e);
        //                             return;
        //                         }
        //                     }
        //                 }
        //                 Err(e) => {
        //                     eprintln!("Error opening serial port: {:?}", e);
        //                 }
        //             }
        //         });
        //     }
        // });
    // }
}

fn schindler_page(_ui: &AppWindow, 
            _ui_weak: slint::Weak<AppWindow>,
            _mqtt_worker: &MqttWorker) {
    
    // TODO: Implement Schindler Page
}

fn schindler_double_page(ui: &AppWindow, 
    ui_weak: slint::Weak<AppWindow>,
    _mqtt_worker: &MqttWorker) {

    {
        let ui_weak = ui_weak.clone();
        ui.on_back_to_menu_page(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_page(Page::Menu);
            }
        });
    }
}

fn schindler_test_page(ui: &AppWindow, 
    ui_weak: slint::Weak<AppWindow>) {


    // let mut mqttoptions = MqttOptions::new("test", "mqtt.lift-online.eu", 8883);
    // mqttoptions.set_keep_alive(Duration::from_secs(5));

    // let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // let mqtt_worker = MqttWorker::new(client, eventloop);


    let ui_weak = ui_weak.clone();  

}



fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    
    
    // Create the MQTT worker (connects to mqtt.lift-online.eu)
    let id = "hfguh82ge1ugdbflb23r32";
    let host = "mqtt.lift-online.eu";
    let port = 1883;
    let username = "ali";
    let password = "ooxeej0J";
    
    let mut mqtt_options = MqttOptions::new(id, host, port);
    mqtt_options.clean_start();

    mqtt_options.set_credentials(username, password);
    mqtt_options.set_keep_alive(Duration::from_secs(10));

    let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

    // let mqtt_worker = create_mqtt_worker(&ui, ui_weak.clone());
    let mqtt_worker = MqttWorker::new(&ui, client, eventloop);



    // Navigation Logic
    navigation(&ui, ui_weak.clone());

    // Pages
    otis_page(&ui, ui_weak.clone(), &mqtt_worker);
    schindler_page(&ui, ui_weak.clone(), &mqtt_worker);
    schindler_double_page(&ui, ui_weak.clone(), &mqtt_worker);
    schindler_test_page(&ui, ui_weak.clone());
    
    ui.run()?;

    mqtt_worker.join().unwrap();
    Ok(())
}




// pub fn send_and_receive_serial(send_data: String) -> String {
//     let response_string = Arc::new(Mutex::new(String::new())); // Shared and safe across threads
//     let response_clone = Arc::clone(&response_string); // Clone for the thread

//     thread::spawn(move || {
//         match comport::open_com_port() {
//             Ok(mut port) => {
//                 if let Err(e) = comport::write_to_com_port(&mut *port, &send_data) {
//                     eprintln!("Error writing to COM port: {}", e);
//                     return;
//                 }
//                 match comport::continuous_read(&mut *port) {
//                     Ok(response) => {
//                         let _ = invoke_from_event_loop(move || {
//                             if let Ok(mut data) = response_clone.lock() {
//                                 data.push_str(&response); // Safe modification
//                             }
//                         });
//                     }
//                     Err(e) => {
//                         eprintln!("Error reading from COM port: {}", e);
//                         return;
//                     }
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error opening serial port: {:?}", e);
//             }
//         }
//     });
    

//     // Return the final response safely
//     response_string.lock().unwrap().clone()
// }
