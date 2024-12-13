use serde::{Deserialize, Serialize};
use crate::AppConfig;

#[derive(Deserialize, Serialize)]
struct EmailUser {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Address")]
    address: String,
}

#[derive(Deserialize, Serialize)]
pub struct WebhookMessage {
    #[serde(rename = "ID")]
    pub(crate) id: String,
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
pub struct Attachment {
    #[serde(rename = "ContentType")]
    content_type: String,
    #[serde(rename = "FileName")]
    file_name: String,
    #[serde(rename = "PartID")]
    pub(crate) part_id: String,
    #[serde(rename = "Size")]
    size: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessageDetails {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "Attachments")]
    pub(crate) attachments: Vec<Attachment>,
}

impl MessageDetails {
    pub fn get_camera_name(&self) -> String {
        let camera_name_line = self
            .text
            .lines()
            .find(|line| line.starts_with("Alarm Camera Name:"))
            .unwrap();
        camera_name_line.split(":").collect::<Vec<&str>>()[1]
            .trim()
            .to_string()
    }
}

pub struct ImageData {
    pub(crate) camera_name: String,
    pub(crate) data: bytes::Bytes,
}

pub async fn download_attachment(config: &AppConfig, id: String) -> ImageData {
    println!("Downloading attachment with id: {}", id);
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/message/{}", config.mailpit_url, id))
        .send();

    // parse response as json of MessageDetails
    let message_details: MessageDetails = response.await.unwrap().json().await.unwrap();

    // download first attachment
    let attachment = message_details.attachments.first().unwrap();
    let response = client
        .get(format!(
            "{}/api/v1/message/{}/part/{}",
            config.mailpit_url, id, attachment.part_id
        ))
        .send();

    ImageData {
        camera_name: message_details.get_camera_name(),
        data: response.await.unwrap().bytes().await.unwrap(),
    }
}
