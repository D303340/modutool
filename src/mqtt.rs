use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, Event, EventLoop};

/// Messages sent to the MQTT worker.
pub enum MqttMessage {
    /// Request the worker to quit.
    Quit,
    /// Publish a message with a topic and payload.
    Publish {
        topic: String,
        payload: String,
        retain: bool,
    },
}

/// The MQTT worker runs in its own thread.
pub struct MqttWorker {
    /// Channel for sending commands to the MQTT worker.
    pub command_tx: UnboundedSender<MqttMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl MqttWorker {
    /// Create a new MQTT worker.
    ///
    /// Note that the UI is not referenced here.
    pub fn new(client: AsyncClient, eventloop: EventLoop) -> Self {
        let (command_tx, receiver) = tokio::sync::mpsc::unbounded_channel();

        let worker_thread = std::thread::spawn(move || {
            // Create a dedicated Tokio runtime for this worker thread.
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(mqtt_worker_loop(receiver, client, eventloop))
        });

        Self {
            command_tx,
            worker_thread,
        }
    }

    /// Gracefully shut down the MQTT worker.
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.command_tx.send(MqttMessage::Quit);
        self.worker_thread.join()
    }
}

/// Asynchronous loop that processes MQTT commands.
async fn mqtt_worker_loop(
    mut receiver: UnboundedReceiver<MqttMessage>,
    client: AsyncClient,
    eventloop: EventLoop,
) {
    // Publish an initial startup message.
    client
        .publish("ui", QoS::ExactlyOnce, false, "1")
        .await
        .unwrap();

    // Spawn a separate task to process incoming MQTT events.
    let client_task = tokio::task::spawn(mqtt_client(client.clone(), eventloop));

    // Process commands sent via the channel.
    while let Some(message) = receiver.recv().await {
        match message {
            MqttMessage::Publish { topic, payload, retain } => {
                client
                    .publish(topic, QoS::AtLeastOnce, retain, payload)
                    .await
                    .unwrap();
            }
            MqttMessage::Quit => {
                // Abort the MQTT event task and exit the loop.
                client_task.abort();
                break;
            }
        }
    }
}

/// Asynchronous task that listens for MQTT events.
async fn mqtt_client(
    client: AsyncClient,
    mut eventloop: EventLoop,
) -> tokio::io::Result<()> {
    // Subscribe to the "counter" topic.
    client.subscribe("counter", QoS::AtMostOnce).await.unwrap();

    loop {
        let event = eventloop.poll().await;
        match &event {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                println!(
                    "Incoming MQTT publish: topic = {:?}, payload = {:?}",
                    p.topic, p.payload
                );
                // If needed, parse the payload and do further processing here.
                if let Ok(topic_str) = std::str::from_utf8(&p.topic) {
                    if topic_str == "counter" {
                        if let Ok(payload_str) = std::str::from_utf8(&p.payload) {
                            if let Ok(counter) = payload_str.parse::<i32>() {
                                println!("Received counter update: {}", counter);
                                // Instead of updating the UI here,
                                // you could send this event back to main if desired.
                            }
                        }
                    }
                }
            }
            Ok(other) => {
                println!("MQTT event: {:?}", other);
            }
            Err(e) => {
                println!("MQTT error: {:?}", e);
            }
        }
    }
}
