use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub api_key: String,

    // S3 credentials
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_region: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok(); // Optional: load .env automatically

        let port = std::env::var("PORT")
            .expect("PORT must be set")
            .parse::<u16>()
            .expect("PORT must be a valid u16");

        let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".into())
            .parse()
            .expect("JWT_EXPIRATION_HOURS must be an integer");

        Self {
            port,
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration_hours,
            api_key: std::env::var("API_KEY").expect("API_KEY must be set"),

            s3_access_key_id: std::env::var("S3_ACCESS_KEY_ID")
                .expect("S3_ACCESS_KEY_ID must be set"),
            s3_secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY")
                .expect("S3_SECRET_ACCESS_KEY must be set"),
            s3_region: std::env::var("S3_REGION").expect("S3_REGION must be set"),
            s3_endpoint: std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set"),
            s3_bucket: std::env::var("S3_BUCKET").expect("S3_BUCKET must be set"),
        }
    }
}
