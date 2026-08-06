use std::{env, time::Duration};

const DEFAULT_USER_ID: &str = "9e20b80e-5143-4f77-ad9d-3637ca4c3eba";

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

        let api_endpoint = env::var("API_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:3000/sessions/add".to_string());

        Ok(Self {
            api_endpoint: normalize_upload_endpoint(api_endpoint),
            api_key: required_env("API_KEY")?,
            user_id: parse_user_id(
                env::var("RECORDER_USER_ID")
                    .or_else(|_| env::var("USER_ID"))
                    .unwrap_or_else(|_| DEFAULT_USER_ID.to_string()),
            )?,
            midi_port_name: env::var("MIDI_PORT_NAME").unwrap_or_else(|_| "Roland".to_string()),
            idle_timeout: Duration::from_secs(parse_env("IDLE_TIMEOUT_SECONDS", 5)?),
            live_ws_endpoint: optional_live_endpoint(),
        })
    }
}

fn normalize_upload_endpoint(endpoint: String) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if endpoint.ends_with("/sessions") {
        format!("{endpoint}/add")
    } else {
        endpoint.to_string()
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
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{key} must be set in recorder/.env").into())
}

fn parse_user_id(value: String) -> Result<String, Box<dyn std::error::Error>> {
    let valid = value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        });

    if valid {
        Ok(value)
    } else {
        Err("RECORDER_USER_ID must be a valid user UUID from the backend".into())
    }
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
