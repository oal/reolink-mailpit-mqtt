use dotenvy::dotenv;

pub const INTEGRATION_NAME: &str = "Reolink Mailpit";
pub const INTEGRATION_IDENTIFIER: &str = "reolink-mailpit";

#[derive(Clone)]
pub struct AppConfig {
    pub(crate) mailpit_url: String,
    pub(crate) mqtt_host: String,
    pub(crate) mqtt_port: u16,
}

impl AppConfig {
    pub fn from_env() -> AppConfig {
        dotenv().expect(".env file not found");

        let mailpit_url = std::env::var("MAILPIT_URL").expect("MAILPIT_URL not set");
        let mqtt_host = std::env::var("MQTT_HOST").expect("MQTT_HOST not set");
        let mqtt_port = std::env::var("MQTT_PORT").expect("MQTT_PORT not set");
        let mqtt_port = mqtt_port
            .parse::<u16>()
            .expect("MQTT_PORT is not a valid port");

        AppConfig {
            mailpit_url,
            mqtt_host,
            mqtt_port,
        }
    }
}
