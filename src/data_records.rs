use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InputRecord {
    #[serde(rename = "type")]
    oper_type: String,
    client: u16,
    tx: u32,
    amount: f32,
}

#[derive(Debug, Serialize)]
pub struct OutputRecord {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}
