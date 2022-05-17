use crate::error::Error;

use async_trait::async_trait;

pub mod svt;

#[async_trait]
pub trait Spider: Send + Sync {
    type Item;

    fn start_urls(&self) -> Vec<String>;

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}
