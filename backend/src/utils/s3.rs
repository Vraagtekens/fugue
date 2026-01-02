use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_s3::{
    Client,
    config::{Credentials, Region, endpoint::Endpoint},
    primitives::ByteStream,
};

use crate::config::Config;

#[derive(Clone)]
pub struct S3Manager {
    client: Client,
    bucket: String,
}

impl S3Manager {
    pub async fn new(config: &Config) -> Self {
        // Build AWS credentials
        let credentials = Credentials::new(
            &config.s3_access_key_id,
            &config.s3_secret_access_key,
            None,
            None, // expiration
            "s3-manager",
        );

        // Build region
        let region = Region::new(config.s3_region.clone());

        // Load shared config
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(credentials)
            .endpoint_url(&config.s3_endpoint)
            .load()
            .await;

        let client = Client::new(&shared_config);

        Self {
            client,
            bucket: config.s3_bucket.clone(),
        }
    }

    /// Upload a file to S3 and return the file URL
    pub async fn add_file(
        &self,
        key: &str,
        file: Vec<u8>,
        _key_prefix: Option<&str>,
    ) -> Result<String, aws_sdk_s3::Error> {
        let body = ByteStream::from(file);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .send()
            .await?;

        // Return URL matching your custom endpoint
        Ok(format!("{}/{}", self.bucket, key))
    }

    pub async fn get_file(&self, key: &str) -> Result<ByteStream, aws_sdk_s3::Error> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(resp.body)
    }
}
