use rumqttc::v5::{
    AsyncClient, 
    Event, 
    EventLoop, 
    MqttOptions
};

use rumqttc::v5::mqttbytes::v5::{LastWill, Packet};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::{TlsConfiguration, Transport};
use slint::SharedString;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::fs;
use std::time::Duration;
use std::thread;
use std::env;

use crate::AppWindow;

// Re-export so main.rs can see this.
pub enum MqttMessage {
    Quit,
    Publish {
        topic: SharedString,
        payload: SharedString,
    },
}

pub struct MqttWorker {
    pub channel: UnboundedSender<MqttMessage>,
    worker_thread: thread::JoinHandle<()>,
}

impl MqttWorker {
    pub fn join(self) -> thread::Result<()> {
        let _ = self.channel.send(MqttMessage::Quit);
        self.worker_thread.join()
    }
}

/// The main worker loop that processes messages from the channel and runs the event loop.
/// It also spawns a background `mqtt_client` task to handle incoming events.
pub async fn mqtt_worker_loop(
    mut r: UnboundedReceiver<MqttMessage>,
    handle: slint::Weak<AppWindow>,
    client: AsyncClient,
    eventloop: EventLoop,
) {
    // Example: Publish something initially
    client
        .publish("ui", QoS::ExactlyOnce, false, "1")
        .await
        .unwrap();

    // Spawn the background task for handling MQTT events
    let client_task = tokio::task::spawn(mqtt_client(handle.clone(), client.clone(), eventloop));

    // Handle commands from the channel (Publish, Quit, etc.)
    loop {
        if let Some(msg) = r.recv().await {
            match msg {
                MqttMessage::Publish { topic, payload } => {
                    client
                        .publish(topic.to_string(), QoS::AtLeastOnce, false, payload.to_string())
                        .await
                        .unwrap();
                }
                MqttMessage::Quit => {
                    client_task.abort();
                    break;
                }
            }
        }
    }
}

/// A background task that polls the event loop for incoming MQTT messages
/// and updates the UI if it sees messages on the `test/sch/output` topic.
async fn mqtt_client(
    handle: slint::Weak<AppWindow>,
    client: AsyncClient,
    mut eventloop: EventLoop,
) -> tokio::io::Result<()> {
    client.subscribe("test/sch/output", QoS::AtMostOnce).await.unwrap();

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                println!("Incoming = {:?}, {:?}", p.topic, p.payload);

                let topic_str = std::str::from_utf8(&p.topic).unwrap_or("");
                let payload_str = std::str::from_utf8(&p.payload).unwrap_or("");

                if topic_str == "test/sch/output" {
                    if let Ok(payload_value) = payload_str.parse::<i32>() {
                        let string_payload = SharedString::from(payload_value.to_string());

                        println!("Received payload: {:?}", string_payload);

                        let _ = handle.upgrade_in_event_loop(|ui| {
                            ui.set_console_output(string_payload);
                        });
                    }
                }
            }
            Ok(other_event) => {
                println!("Other event: {:?}", other_event);
            }
            Err(e) => {
                println!("Error from eventloop: {:?}\n\n", e);
            }
        }
    }
}

/// Creates the MQTT connection with optional TLS and credentials,
/// and spawns a background thread to run the `mqtt_worker_loop`.
pub fn create_mqtt_worker(ui: &AppWindow, ui_weak: slint::Weak<AppWindow>) -> MqttWorker {
    // 1) Configure hostname and port
    //    If you want a non-TLS connection, use port 1883 and skip the TLS code below.
    let mut mqttoptions = MqttOptions::new("my_client_id", "mqtt.lift-online.eu", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // 2) (Optional) Load CA certificate if using TLS
    //    If you have a CA cert, read it here. Otherwise, remove or comment out these lines.
    let path = env::current_dir().unwrap();
    // let ca_certificate: Vec<u8> = include_bytes!(".mqtt-ca.crt").to_vec();

    // 3) (Optional) Set username/password if required
    mqttoptions.set_credentials("ali", "ooxeej0J");

    // 5) Last Will (optional)
    let will = LastWill::new("ui", "0", QoS::AtMostOnce, false, None);
    mqttoptions.set_last_will(will);

    // Create the client + eventloop pair
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // Create a channel for sending Publish/Quit messages to the worker
    let (channel, r) = tokio::sync::mpsc::unbounded_channel();

    // We’ll spawn a dedicated thread for the async runtime
    let handle_weak = ui_weak.clone();
    let worker_thread = std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(mqtt_worker_loop(r, handle_weak, client, eventloop))
    });

    MqttWorker {
        channel,
        worker_thread,
    }
}