use slint::{Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;

use helpers::mqtt::MqttMessage;
use rumqttc::v5::{ MqttOptions, AsyncClient};
use tokio::time::Duration;

use serde_json::from_str;

use crate::slint_ui::*;

#[path = "../helpers"]
mod helpers {
    pub mod mqtt;
    pub mod convert;
}

#[path = "../models"]
mod models {
    pub mod json;
}


pub async fn schindler_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>){
    let ui_weak = ui_weak.clone();


    // MQTT client setup
    let id = "hfguh82ge1ugdbflb23r32";
    let host = "localhost";
    let port = 1883;
    // let username = "ali";
    // let password = "ooxeej0J";
    
    let mut mqtt_options = MqttOptions::new(id, host, port);
    mqtt_options.clean_start();

    // mqtt_options.set_credentials(username, password);
    mqtt_options.set_keep_alive(Duration::from_secs(10));

    let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

    let mut mqtt_worker = helpers::mqtt::MqttWorker::new(&ui, client, eventloop);

  

    // Optional: Take events if you want to process them
    if let Some(mut events) = mqtt_worker.take_events() {
        let ui_weak = ui_weak.clone();
        
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                match event {
                    helpers::mqtt::MqttEvent::Incoming { topic: _, payload } => {
                        let _result = ui_weak.upgrade_in_event_loop(move |ui| {
                            match from_str::<models::json::SchindlerTestData>(&payload) {
                                Ok(schindler) => {
                                    let messages : ModelRc<console_output> =  ui.global::<SchindlerPageLogic>().get_output_message();
                                    let just_time = schindler.time.split_whitespace().nth(1).unwrap_or(&schindler.time);


                                    let mut items: Vec<console_output> = messages.iter().collect();

                                    // Add your new element:
                                    let new_message = format!("{}", schindler.message);

                                    items.push(console_output {
                                        time: just_time.into(),
                                        message: SharedString::from(new_message),
                                    });

                                    const MAX_ITEMS: usize = 300;
                                    if items.len() > MAX_ITEMS {
                                        // Remove the extra elements from the beginning.
                                        items.drain(0..(items.len() - MAX_ITEMS));
}

                                    // Create a new VecModel from the updated items:
                                    let new_model = VecModel::from(items);

                                    // Wrap the new VecModel in an Rc and update the UI:
                                    ui.global::<SchindlerPageLogic>().set_output_message(ModelRc::from(Rc::new(new_model)));
                                },
                                Err(e) => eprintln!("Failed to parse JSON: {}", e),
                            }
                        });
                    }
                }
            }
        });


        
        let numbers: ModelRc<f32> = ui.global::<SchindlerPageLogic>().get_graph_data();

        // KEYPAD
        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_keypad_enter(move |keypad_value| {    
            println!("\n\n\nVALUE OF THE KEYDPAD: {}\n\n\n", keypad_value);
            
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("c/sch/input"), 
                payload: keypad_value // Clone the SharedString to avoid moving it
            });
        });
        


        
        
        // COMMANDS
        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_Send_Command(move |command| {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from(command) // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_Send_Command_Number(move |command, value| {

            let mut combined_string = String::from(command);
            combined_string.push_str(&format!("{}", value));

            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from(combined_string) // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_Send_Command_HEX(move |command, value| {
            // convert to hex
            let hex_number = format!("{:X}", value);
            
            //combine string: SYS_SIM_START:= with hex_number
            let mut combined_string = String::from(command);
            combined_string.push_str(&hex_number);

            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from(combined_string) // Clone the SharedString to avoid moving it
            });
        });
        
    }
}