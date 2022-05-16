use async_trait::async_trait;

pub mod svt;

#[async_trait]
pub trait Spider: Send + Sync {
    fn start_urls(&self) -> Vec<String>;
    async fn scrape(&self, url: String) -> Result<(String, String), String>;
    async fn process(&self, url_item: (String, String)) -> Result<(), String>;
}
