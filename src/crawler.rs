use crate::{
    spiders::Spider,
};
use std::sync::Arc;

pub struct Crawler {}

impl Crawler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(
        &self,
        spider: Arc<dyn Spider>,
    ) {
        eprintln!("crawler: run");
        for url in spider.start_urls() {
            let item = spider
                .scrape(url)
                .await
                .expect("crawler: scraping url");
            let _ = spider.process(item).await;
        }
    }
}
