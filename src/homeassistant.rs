use serde::Serialize;

#[derive(Serialize)]
pub struct Device {
    pub(crate) identifiers: Vec<&'static str>,
    pub(crate) name: &'static str,
}

#[derive(Serialize)]
pub struct ConfigMessage {
    pub(crate) name: String,
    pub(crate) unique_id: String,
    pub(crate) object_id: String,
    pub(crate) image_topic: String,
    pub(crate) device: Device,
    // pub state_topic: String,
}
