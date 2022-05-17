use crate::{
    spiders::Spider,
};
use std::sync::Arc;

pub struct Crawler {}

impl Crawler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run<T: Send + Sync>(
        &self,
        spider: Arc<dyn Spider<Item = T>>,
    ) {
        log::debug!("crawler: run");
        let mut urls_to_visit = spider.start_urls();
        for url in urls_to_visit.iter() {
            let (items, new_urls) = spider
                .scrape(url.to_string())
                .await
                .expect("crawler: scraping url");
            for item in items {
                spider.process(item).await.expect("crawler: processing");
            }

            for new_url in new_urls.into_iter() {
                urls_to_visit.push(new_url);

            }
        }
    }
}
