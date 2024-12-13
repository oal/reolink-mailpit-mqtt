use crate::mailpit::{download_attachment, WebhookMessage};
use crate::sensor::MqttImageSensor;
use crate::{AppConfig};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use http::StatusCode;

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Point Mailpit webhook to /email-webhook"
}

async fn email_webhook(
    State(config): State<AppConfig>,
    Json(message): Json<WebhookMessage>,
) -> StatusCode {
    let image_data = download_attachment(&config, message.id.clone()).await;

    let mut sensor = MqttImageSensor::new(&config);
    sensor.configure(&image_data).await;
    sensor.send_image(image_data).await;

    StatusCode::OK
}

pub async fn run(config: AppConfig) {
    let app = Router::new()
        .route("/", get(root))
        .route("/email-webhook", post(email_webhook))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8026").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
