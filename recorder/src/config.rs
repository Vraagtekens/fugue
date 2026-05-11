use std::{env, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_endpoint: String,
    pub api_key: String,
    pub user_id: String,
    pub midi_port_name: String,
    pub idle_timeout: Duration,
    pub live_ws_endpoint: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            api_endpoint: env::var("API_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3000/sessions/add".to_string()),
            api_key: required_env("API_KEY")?,
            user_id: env::var("RECORDER_USER_ID").or_else(|_| env::var("USER_ID"))?,
            midi_port_name: env::var("MIDI_PORT_NAME").unwrap_or_else(|_| "Roland".to_string()),
            idle_timeout: Duration::from_secs(parse_env("IDLE_TIMEOUT_SECONDS", 5)?),
            live_ws_endpoint: optional_live_endpoint(),
        })
    }
}

fn optional_live_endpoint() -> Option<String> {
    match env::var("LIVE_WS_ENDPOINT") {
        Ok(value) if value.eq_ignore_ascii_case("off") => None,
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => Some("ws://localhost:3000/sessions/live/record".to_string()),
    }
}

fn required_env(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(key).map_err(|_| format!("{key} must be set").into())
}

fn parse_env<T>(key: &str, default: T) -> Result<T, Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    match env::var(key) {
        Ok(value) => Ok(value.parse()?),
        Err(_) => Ok(default),
    }
}
