use std::time::Duration;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use rumqttc::MqttOptions;
use rumqttc::Event::Incoming;
use rumqttc::PubAck;
use dotenvy::dotenv;
use crate::homeassistant::{ConfigMessage, Device};

mod homeassistant;

const INTEGRATION_NAME: &str = "Reolink Mailpit";
const INTEGRATION_IDENTIFIER: &str = "reolink-mailpit";

#[derive(Clone)]
struct AppConfig {
    mailpit_url: String,
    mqtt_host: String,
    mqtt_port: u16,
}

impl AppConfig {
    fn from_env() -> AppConfig {
        dotenv().expect(".env file not found");

        let mailpit_url = std::env::var("MAILPIT_URL").expect("MAILPIT_URL not set");
        let mqtt_host = std::env::var("MQTT_HOST").expect("MQTT_HOST not set");
        let mqtt_port = std::env::var("MQTT_PORT").expect("MQTT_PORT not set");
        let mqtt_port = mqtt_port.parse::<u16>().expect("MQTT_PORT is not a valid port");

        AppConfig {
            mailpit_url,
            mqtt_host,
            mqtt_port,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let config = AppConfig::from_env();
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let mut app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/email-webhook", post(email_webhook))
        .with_state(config);


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8026").await.unwrap();



    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Point Mailpit webhook to /email-webhook"
}

#[derive(Deserialize, Serialize)]
struct EmailUser {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Address")]
    address: String,
}

#[derive(Deserialize, Serialize)]
struct WebhookMessage {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(rename = "Read")]
    read: bool,
    #[serde(rename = "From")]
    from: EmailUser,
    #[serde(rename = "To")]
    to: Vec<EmailUser>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "Created")]
    created: String,
    #[serde(rename = "Size")]
    size: i32,
    #[serde(rename = "Attachments")]
    attachments: i32,
    #[serde(rename = "Snippet")]
    snippet: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Attachment {
    #[serde(rename = "ContentType")]
    content_type: String,
    #[serde(rename = "FileName")]
    file_name: String,
    #[serde(rename = "PartID")]
    part_id: String,
    #[serde(rename = "Size")]
    size: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct MessageDetails {
    #[serde(rename = "Attachments")]
    attachments: Vec<Attachment>,
}

async fn download_attachment(config: &AppConfig, id: String) -> bytes::Bytes {
    println!("Downloading attachment with id: {}", id);
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/message/{}", config.mailpit_url, id))
        .send();

    // parse response as json of MessageDetails
    let message_details: MessageDetails = response.await.unwrap().json().await.unwrap();
    //
    println!("{:?}", message_details);

    // download first attachment
    let attachment = message_details.attachments.first().unwrap();
    let file_name = attachment.file_name.clone();
    let response = client
        .get(format!("{}/api/v1/message/{}/part/{}", config.mailpit_url, id, attachment.part_id)).send();

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_name)
        .await
        .unwrap();

    let bytes = response.await.unwrap().bytes().await.unwrap();
    // tokio::io::AsyncWriteExt::write(&mut file, &bytes).await.unwrap();
    bytes
}

async fn email_webhook(State(config): State<AppConfig>, Json(message): Json<WebhookMessage>) -> StatusCode {
    let bytes = download_attachment(&config, message.id).await;

    let mut mqtt_options = MqttOptions::new(INTEGRATION_IDENTIFIER, config.mqtt_host, config.mqtt_port);

    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = rumqttc::Client::new(mqtt_options, 10);
    let config_topic = format!("{}/config", INTEGRATION_IDENTIFIER);
    let config_message = ConfigMessage {
        name: "movement".to_string(),
        device_class: "timestamp".to_string(),
        // unit_of_measurement: "date".to_string(),
        state_topic: "reolink-mailpit/sensor/movement/state".to_string(),
        unique_id: "reolink-mailpit-movement".to_string(),
        object_id: "reolink-mailpit-movement".to_string(),
        device: Device {
            identifiers: vec![INTEGRATION_IDENTIFIER],
            name: INTEGRATION_NAME,
        },
    };

    let config_serialized = serde_json::to_string(&config_message).unwrap();
    // TODO TODO from here...

    client.publish(config_topic, rumqttc::QoS::AtLeastOnce, false, config_serialized).expect("Failed to publish config message");
    // request body as string
    // let payload = serde_json::to_string(&message).unwrap();
    // println!("{}", payload);
    //
    // // serialize json
    // // print body
    // // write body to file
    // let mut file = tokio::fs::OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open("emails.txt")
    //     .await
    //     .unwrap();
    // tokio::io::AsyncWriteExt::write(&mut file, payload.as_bytes()).await.unwrap();
    StatusCode::OK
}
