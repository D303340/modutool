use rumqttc::v5::{AsyncClient, Event, EventLoop, MqttOptions};

use rumqttc::v5::mqttbytes::v5::{LastWill, Packet};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::{TlsConfiguration, Transport};
use slint::SharedString;
use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::AppWindow;
use slint::ComponentHandle;

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
        .publish("test/ui", QoS::ExactlyOnce, false, "1")
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
                        .publish(
                            topic.to_string(),
                            QoS::AtLeastOnce,
                            false,
                            payload.to_string(),
                        )
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

    client
        .subscribe("test/sch/output", QoS::AtMostOnce)
        .await
        .unwrap();
    loop {
        let event = eventloop.poll().await;

        match &event {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                let payload_str = String::from_utf8_lossy(&p.payload);
                println!("Incoming = {:?}, {:?}\n\n", p.topic, payload_str);
            }
            Ok(v) => {
                println!("Event = {v:?}");
            }
            Err(e) => {
                println!("Error = {e:?}\n\n");
            }
        }
    }
}

/// Creates the MQTT connection with optional TLS and credentials,
/// and spawns a background thread to run the `mqtt_worker_loop`.
impl MqttWorker {
    pub fn new(ui: &AppWindow, client: AsyncClient, eventloop: EventLoop) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();

        let worker_thread = std::thread::spawn({
            let handle_weak = ui.as_weak();

            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(mqtt_worker_loop(r, handle_weak, client, eventloop))
            }
        });

        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(MqttMessage::Quit);
        self.worker_thread.join()
    }
}

