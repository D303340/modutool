// src/helpers/mqtt.rs

//! A simple, flexible MQTT helper using `rumqttc` v5.
//!
//! This module defines:
//! - `MqttMessage`: commands you can send _into_ the worker (Publish, Subscribe, Unsubscribe, Quit).
//! - `MqttEvent`: events coming _out_ of the worker when the broker publishes a message.
//! - `MqttWorker`: a self‐contained MQTT client that runs on its own thread with a Tokio runtime.
//!
//! # Usage Example
//! ```ignore
//! use helpers::mqtt::{MqttWorker, MqttMessage, MqttEvent};
//! use rumqttc::v5::{MqttOptions, AsyncClient, EventLoop};
//! use tokio::runtime::Runtime;
//!
//! fn main() {
//!     // 1) Build your own AsyncClient + EventLoop first:
//!     let mut opts = MqttOptions::new("my_client_id", "localhost", 1883);
//!     opts.set_keep_alive(std::time::Duration::from_secs(10));
//!     let (client, eventloop) = AsyncClient::new(opts, 10);
//!
//!     // 2) Create the worker, subscribe initially to "output" (or any topics):
//!     let mut worker = MqttWorker::new(client, eventloop, vec!["output".into()]);
//!
//!     // 3) Take the incoming‐message receiver
//!     let mut incoming = worker.take_event_receiver().unwrap();
//!
//!     // 4) Spawn a Tokio task to process incoming events:
//!     let rt = Runtime::new().unwrap();
//!     rt.handle().spawn(async move {
//!         while let Some(evt) = incoming.recv().await {
//!             match evt {
//!                 MqttEvent::Incoming { topic, payload } => {
//!                     println!("Received on `{}` → `{}`", topic, payload);
//!                 }
//!             }
//!         }
//!     });
//!
//!     // 5) Publish whenever you need:
//!     let _ = worker.publish("output", "Hello from Rust!");
//!
//!     // 6) Subscribe to more topics at runtime if needed:
//!     let _ = worker.subscribe("another/topic");
//!
//!     // 7) … do other work here …
//!     std::thread::sleep(std::time::Duration::from_secs(5));
//!
//!     // 8) When shutting down, call `quit()` to cleanly stop the MQTT thread:
//!     let _ = worker.quit();
//! }
//! ```




use rumqttc::v5::{
    AsyncClient,
    Event,
    EventLoop,
    MqttOptions,
    mqttbytes::v5::Packet,
    mqttbytes::QoS,
};
use std::thread;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Commands you send **into** the MQTT worker.
#[derive(Debug)]
pub enum MqttMessage {
    /// Subscribe to `topic` (QoS = AtMostOnce).
    Subscribe(String),

    /// Unsubscribe from `topic`.
    Unsubscribe(String),

    /// Publish `payload` to `topic` (QoS = AtLeastOnce, no retain).
    Publish {
        topic: String,
        payload: String,
    },

    /// Disconnect and stop the background worker.
    Quit,
}

/// Events emitted **by** the MQTT worker whenever a PUBLISH is received.
#[derive(Debug)]
pub enum MqttEvent {
    /// A message was published on `topic` with UTF‐8 `payload`.
    Incoming {
        topic: String,
        payload: String,
    },
}

/// `MqttWorker` spawns a Tokio runtime on its own thread. You can:
/// 1) Send `MqttMessage` commands (Publish / Subscribe / Unsubscribe / Quit).
/// 2) Receive every incoming PUBLISH as `MqttEvent::Incoming { topic, payload }`.
pub struct MqttWorker {
    /// Send commands (Publish / Subscribe / etc.) here.
    pub channel: UnboundedSender<MqttMessage>,

    /// A one‐time receiver for incoming `MqttEvent`. Call `take_event_receiver()` once.
    pub events: Option<UnboundedReceiver<MqttEvent>>,

    // Keep the thread handle so we can join it when quitting.
    worker_thread: thread::JoinHandle<()>,
}

impl MqttWorker {
    /// Spawn a new `MqttWorker`.
    ///
    /// - `client`:    your `AsyncClient` from `AsyncClient::new(...)`.
    /// - `eventloop`: the accompanying `EventLoop`.
    /// - `initial_subs`: a `Vec<String>` of topics to subscribe to immediately.
    pub fn new(
        client: AsyncClient,
        mut eventloop: EventLoop,
        initial_subs: Vec<String>,
    ) -> Self {
        // 1) Create unbounded channels for commands & events:
        let (cmd_tx, mut cmd_rx) = unbounded_channel::<MqttMessage>();
        let (evt_tx, evt_rx) = unbounded_channel::<MqttEvent>();

        // 2) Spawn an OS thread with its own Tokio runtime:
        let handle = thread::spawn(move || {
            // Build a simple multi‐threaded Tokio runtime:
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                // A) Immediately subscribe to all initial topics:
                for topic in initial_subs {
                    let _ = client.subscribe(topic, QoS::AtMostOnce).await;
                }

                // B) Main loop: await either a command or an incoming MQTT event:
                loop {
                    tokio::select! {
                        // 1) COMMAND arrived on cmd_rx:
                        maybe_cmd = cmd_rx.recv() => {
                            match maybe_cmd {
                                Some(MqttMessage::Subscribe(topic)) => {
                                    let _ = client
                                            .subscribe(topic, QoS::AtMostOnce)
                                            .await;
                                }
                                Some(MqttMessage::Unsubscribe(topic)) => {
                                    let _ = client
                                            .unsubscribe(topic)
                                            .await;
                                }
                                Some(MqttMessage::Publish { topic, payload }) => {
                                    let _ = client
                                            .publish(topic, QoS::AtLeastOnce, false, payload)
                                            .await;
                                }
                                Some(MqttMessage::Quit) | None => {
                                    // Either explicit Quit, or channel closed: break out.
                                    break;
                                }
                            }
                        }

                        // 2) NEW MQTT EVENT (e.g. a PUBLISH packet)
                        mqtt_event = eventloop.poll() => {
                            if let Ok(Event::Incoming(Packet::Publish(p))) = mqtt_event {
                                let topic_str = String::from_utf8_lossy(&p.topic).to_string();
                                let payload_str = String::from_utf8_lossy(&p.payload).to_string();
                                let _ = evt_tx.send(MqttEvent::Incoming {
                                    topic: topic_str,
                                    payload: payload_str,
                                });
                            }
                            // You can ignore or log other events/errors here if desired.
                        }
                    } // end select!
                }

                // C) Clean up: disconnect the client, then let thread exit
                let _ = client.disconnect().await;
            });
        });

        MqttWorker {
            channel: cmd_tx,
            events: Some(evt_rx),
            worker_thread: handle,
        }
    }

    /// Publish `payload` to `topic` (QoS = AtLeastOnce, no retain).
    pub fn publish(
        &self,
        topic: impl Into<String>,
        payload: impl Into<String>,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<MqttMessage>> {
        self.channel.send(MqttMessage::Publish {
            topic: topic.into(),
            payload: payload.into(),
        })
    }

    /// Subscribe to `topic` (QoS = AtMostOnce).
    pub fn subscribe(
        &self,
        topic: impl Into<String>,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<MqttMessage>> {
        self.channel.send(MqttMessage::Subscribe(topic.into()))
    }

    /// Unsubscribe from `topic`.
    pub fn unsubscribe(
        &self,
        topic: impl Into<String>,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<MqttMessage>> {
        self.channel
            .send(MqttMessage::Unsubscribe(topic.into()))
    }

    /// Take ownership of the receiver for incoming `MqttEvent`. You can only call this once.
    pub fn take_event_receiver(&mut self) -> Option<UnboundedReceiver<MqttEvent>> {
        self.events.take()
    }

    /// Send the `Quit` command, then join the worker thread to shut down cleanly.
    pub fn quit(self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.channel.send(MqttMessage::Quit);
        // Dropping the sender closes the channel, so the select! sees `None`.
        drop(self.channel);
        let _ = self.worker_thread.join();
        Ok(())
    }
}