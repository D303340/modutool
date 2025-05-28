// src/main.rs (or wherever your Slint‐based code lives)

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;
use rumqttc::v5::{MqttOptions, AsyncClient, EventLoop};
use slint::Model;
use tokio::time::Duration;
use serde_json::from_str;

use crate::slint_ui::*;
#[path = "../helpers"]
mod helpers {
    pub mod mqtt;
}
#[path = "../models"]
mod models {
    pub mod json;
}

use helpers::mqtt::{MqttWorker, MqttMessage, MqttEvent};

pub async fn schindler_page(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak = ui_weak.clone();

    // ───────────
    // 1) MQTT client setup
    // ───────────
    let client_id = "hfguh82ge1ugdbflb23r32";
    let host = "localhost";
    let port = 1883;
    // let username = "ali";
    // let password = "ooxeej0J";

    let mut mqtt_options = MqttOptions::new(client_id, host, port);
    mqtt_options.clean_start();
    mqtt_options.set_keep_alive(Duration::from_secs(10));

    // Build AsyncClient + EventLoop
    let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

    // 2) Create the worker, subscribe initially to "output" (or any topics you want at startup)
    let mut mqtt_worker = MqttWorker::new(
        client,
        eventloop,
        vec!["output".to_string()], // initial subscriptions
    );

    // 3) Take the incoming‐message receiver
    if let Some(mut events_rx) = mqtt_worker.take_event_receiver() {
        let ui_weak_clone = ui_weak.clone();

        // 4) Spawn a Tokio task to process incoming messages:
        tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                if let MqttEvent::Incoming { topic: _, payload } = event {
                    let _ = ui_weak_clone.upgrade_in_event_loop(move |ui| {
                        match from_str::<models::json::SchindlerTestData>(&payload) {
                            Ok(schindler) => {
                                let messages: ModelRc<console_output> =
                                    ui.global::<SchindlerPageLogic>().get_output_message();
                                let just_time = schindler
                                    .time
                                    .split_whitespace()
                                    .nth(1)
                                    .unwrap_or(&schindler.time);

                                let mut items: Vec<console_output> = messages.iter().collect();
                                let new_message = format!("{}", schindler.message);

                                items.push(console_output {
                                    time: just_time.into(),
                                    message: SharedString::from(new_message),
                                });

                                const MAX_ITEMS: usize = 300;
                                if items.len() > MAX_ITEMS {
                                    items.drain(0..(items.len() - MAX_ITEMS));
                                }

                                let new_model = VecModel::from(items);
                                ui.global::<SchindlerPageLogic>()
                                    .set_output_message(ModelRc::from(Rc::new(new_model)));
                            }
                            Err(e) => eprintln!("Failed to parse JSON: {}", e),
                        }
                    });
                }
            }
        });
    }

    // ───────────
    // 5) Hook up Slint callbacks to publish to whatever topics you want
    // ───────────

    // Keypad: publish on "c/sch/input" when the user presses ENTER on the keypad.
    let publish_chan = mqtt_worker.channel.clone();
    ui.global::<SchindlerPageLogic>().on_keypad_enter(move |keypad_value| {
        let _ = publish_chan.send(MqttMessage::Publish {
            topic: "c/sch/input".to_string(),
            payload: keypad_value.to_string(),
        });
    });

    // COMMANDS: publish on "c"
    let publish_chan = mqtt_worker.channel.clone();
    ui.global::<SchindlerPageLogic>().on_Send_Command(move |command| {
        let _ = publish_chan.send(MqttMessage::Publish {
            topic: "c".to_string(),
            payload: command.to_string(),
        });
    });

    // COMMAND + NUMBER: publish on "test/sch/input"
    let publish_chan = mqtt_worker.channel.clone();
    ui.global::<SchindlerPageLogic>().on_Send_Command_Number(move |command, value| {
        let combined = format!("{}{}", command, value);
        let _ = publish_chan.send(MqttMessage::Publish {
            topic: "test/sch/input".to_string(),
            payload: combined,
        });
    });

    // COMMAND + HEX: publish on "test/sch/input"
    let publish_chan = mqtt_worker.channel.clone();
    ui.global::<SchindlerPageLogic>().on_Send_Command_HEX(move |command, value| {
        let hex_str = format!("{:X}", value);
        let combined = format!("{}{}", command, hex_str);
        let _ = publish_chan.send(MqttMessage::Publish {
            topic: "test/sch/input".to_string(),
            payload: combined,
        });
    });

    // 6) You can also subscribe/unsubscribe at runtime if you want:
    //    let _ = mqtt_worker.subscribe("another/topic");
    //    let _ = mqtt_worker.unsubscribe("test/sch/output");

    // (No need to `await` anything here; worker runs on its own thread.)
}
