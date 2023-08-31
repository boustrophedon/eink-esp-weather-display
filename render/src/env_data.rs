use std::path::Path;

use base64::{Engine as _, engine::general_purpose as b64};

#[derive(Debug, Clone)]
pub struct EnvData {
    pub local_timezone: chrono_tz::Tz,
    pub user_agent: String,
    pub tasks_api_key: Vec<u8>,
    pub weather_station: String,
    pub weather_office: String,
    pub weather_gridpoint: String,
}

impl EnvData {
    pub fn from_file(path: &Path) -> EnvData {
        let json_str = std::fs::read_to_string(path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .expect("failed to parse env data json");

        let api_key_data = data["tasks_api_key"].as_str().unwrap();
        let tasks_api_key = b64::STANDARD_NO_PAD.decode(api_key_data).unwrap();
        EnvData {
            local_timezone: data["local_timezone"].as_str().unwrap().parse().unwrap(),
            user_agent: data["user_agent"].as_str().unwrap().into(),
            tasks_api_key: tasks_api_key,
            weather_station: data["weather_station"].as_str().unwrap().into(),
            weather_office: data["weather_office"].as_str().unwrap().into(),
            weather_gridpoint: data["weather_gridpoint"].as_str().unwrap().into(),
        }
    }
}
