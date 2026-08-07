use serde::Deserialize;
use std::{env, error::Error, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub api_key: String,
    pub soundfont_path: PathBuf,

    // S3 credentials
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_region: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        dotenvy::dotenv().ok(); // Optional: load .env automatically

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?;

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".into())
            .parse()?;

        Ok(Self {
            port,
            database_url: required_env("DATABASE_URL")?,
            jwt_secret: required_env("JWT_SECRET")?,
            jwt_expiration_hours,
            api_key: required_env("API_KEY")?,
            soundfont_path: soundfont_path(),

            s3_access_key_id: required_env("S3_ACCESS_KEY")?,
            s3_secret_access_key: required_env("S3_SECRET_ACCESS_KEY")?,
            s3_region: required_env("S3_REGION")?,
            s3_endpoint: required_env("S3_ENDPOINT")?,
            s3_bucket: required_env("S3_BUCKET")?,
        })
    }
}

fn soundfont_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match env::var("SOUNDFONT_PATH").map(PathBuf::from) {
        Ok(path) if path.is_absolute() => path,
        Ok(path) => manifest_dir.join(path),
        Err(_) => manifest_dir.join("assets/YDP-GrandPiano-20160804.sf2"),
    }
}

fn required_env(key: &str) -> Result<String, Box<dyn Error>> {
    env::var(key).map_err(|_| format!("{key} must be set").into())
}
