use std::time::Duration;
use rumqttc::{EventLoop, MqttOptions, PubAck};
use rumqttc::Event::Incoming;
use crate::config::{AppConfig, INTEGRATION_IDENTIFIER, INTEGRATION_NAME};
use crate::homeassistant::{ConfigMessage, Device};
use crate::mailpit::ImageData;

pub struct MqttImageSensor {
    client: rumqttc::AsyncClient,
    event_loop: EventLoop,
}

impl MqttImageSensor {
    pub(crate) fn new(app_config: &AppConfig) -> Self {
        let mqtt_options = MqttImageSensor::options(app_config);
        let (client, event_loop) = rumqttc::AsyncClient::new(mqtt_options, 10);

        Self {
            client,
            event_loop,
        }
    }

    fn options(app_config: &AppConfig) -> MqttOptions {
        let mut mqtt_options = MqttOptions::new(INTEGRATION_IDENTIFIER, &app_config.mqtt_host, app_config.mqtt_port);
        mqtt_options.set_max_packet_size(1000000, 1000000);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        mqtt_options
    }

    fn config_topic(&self) -> String {
        format!("homeassistant/image/{}/config", INTEGRATION_IDENTIFIER)
    }

    fn image_topic(&self) -> String {
        format!("homeassistant/image/{}/image", INTEGRATION_IDENTIFIER)
    }

    pub(crate) async fn configure(&mut self, image_data: &ImageData) {
        let config_message = ConfigMessage {
            name: image_data.camera_name.to_string(),
            unique_id: image_data.camera_name.to_string(),
            object_id: image_data.camera_name.to_string(),
            image_topic: self.image_topic(),
            device: Device {
                identifiers: vec![INTEGRATION_IDENTIFIER],
                name: INTEGRATION_NAME,
            },
        };
        let config_serialized = serde_json::to_string(&config_message).unwrap();
        self.send(&self.config_topic(), config_serialized.into_bytes()).await;
    }

    pub(crate) async fn send_image(&mut self, image_data: ImageData) {
        self.send(&self.image_topic(), image_data.data.into()).await;
    }

    async fn send(&mut self, topic: &str, payload: Vec<u8>) {
        self.client.publish(topic, rumqttc::QoS::AtLeastOnce, false, payload).await.expect("Failed to send MQTT message");
        self.wait_for_ack().await;
    }

    async fn wait_for_ack(&mut self) {
        loop {
            let notification = self.event_loop.poll().await.unwrap();
            match notification {
                Incoming(rumqttc::Packet::PubAck(PubAck { pkid, .. })) => {
                    println!("Received puback for packet id: {}", pkid);
                    break;
                }
                _ => {}
            }
        }
    }
}
