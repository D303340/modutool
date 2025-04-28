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
    let host = "mqtt.lift-online.eu";
    let port = 1883;
    let username = "ali";
    let password = "ooxeej0J";
    
    let mut mqtt_options = MqttOptions::new(id, host, port);
    mqtt_options.clean_start();

    mqtt_options.set_credentials(username, password);
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
        ui.global::<SchindlerPageLogic>().on_keypad_clicked(move |keypad_value| {    
            println!("\n\n\nVALUE OF THE KEYDPAD: {}\n\n\n", keypad_value);

            let num = keypad_value.parse::<f32>().unwrap();

            const MAX_POINTS: usize = 15;

            // 2. downcast to VecModel<f32>
            let vec_model = numbers
                .as_any()
                .downcast_ref::<VecModel<f32>>()
                .expect("graph_data should be a VecModel<f32>");

            // 3. push the new number
            vec_model.push(num);

            // 4. if we're over our limit, pop off the oldest (index 0)
            if vec_model.row_count() > MAX_POINTS {
                vec_model.remove(0);   // keeps the model at MAX_POINTS elements :contentReference[oaicite:0]{index=0}
            }

            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: keypad_value // Clone the SharedString to avoid moving it
            });
        });
        


        
        
        // COMMANDS
        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_SIM_FLOOR_CALL(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("SIM_FLOOR_CALL") // Clone the SharedString to avoid moving it
            });
        });
        
        
        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_LIST_MODE(move |toggled| {
            let payload = if toggled { "List_Mode:=1" } else { "List_Mode:=0" };
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from(payload.to_string()) // Clone the SharedString to avoid moving it
            });
        });
        

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_SYSTEM_INFO(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("SYSTEM_INFO") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_SHOW_VERSION(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("SHOW_VERSION") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_SHOW_SERVICE_STATE(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("SHOW_SERVICE_STATE") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_SHOW_LAST_ERR(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("SHOW_LAST_ERR") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_CLEAR_LAST_ERROR(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("CLEAR_LAST_ERROR") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_READ_SIM_CARD(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("READ_SIM_CARD") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_CTRL_IOSTATUS(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("CTRL_IOSTATUS") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_CAR_IOSTATUS_RQST(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("CAR_IOSTATUS_RQST") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_CAR_DEAD_LOOP(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("CAR_DEAD_LOOP") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_COP_IDENTIFY_HW(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("COP_IDENTIFY_HW") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_COP_IDENTIFY_SW(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("COP_IDENTIFY_SW") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_COP_DEAD_LOOP(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("COP_DEAD_LOOP") // Clone the SharedString to avoid moving it
            });
        });

        let publish_channel = mqtt_worker.channel.clone();
        ui.global::<SchindlerPageLogic>().on_GET_CARLIGHT_STATE(move || {
            let _ = publish_channel.send(MqttMessage::Publish { 
                topic: SharedString::from("test/sch/input"), 
                payload: SharedString::from("GET_CARLIGHT_STATE") // Clone the SharedString to avoid moving it
            });
        });
        
    }
}