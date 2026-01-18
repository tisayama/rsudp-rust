use std::path::Path;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::primitives::ByteStream;
use tracing::info;

pub struct S3Client {
    client: Client,
    bucket: String,
    region: String,
}

impl S3Client {
    pub async fn new(bucket: String, region: String) -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()))
            .load()
            .await;
        let client = Client::new(&config);
        Self {
            client,
            bucket,
            region,
        }
    }

    pub async fn upload_image(&self, local_path: &Path, key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Uploading {} to S3 bucket {} as {}", local_path.display(), self.bucket, key);
        
        let body = ByteStream::from_path(local_path).await?;
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .acl(ObjectCannedAcl::PublicRead)
            .content_type("image/png")
            .send()
            .await?;
            
        let url = format!("https://{}.s3.{}.amazonaws.com/{}", self.bucket, self.region, key);
        info!("Image uploaded successfully: {}", url);
        
        Ok(url)
    }
}
