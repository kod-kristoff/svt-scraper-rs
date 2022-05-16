use async_trait::async_trait;
use std::time::Duration;

pub struct SvtSpider {
    http_client: reqwest::Client,
}

impl SvtSpider {
    pub fn new(_debug: bool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .expect("spiders/svt: Building HTTP client");

        Self { http_client }
    }
}

#[async_trait]
impl super::Spider for SvtSpider {
    fn start_urls(&self) -> Vec<String> {
        vec![self.start_url.clone()]
    }

    async fn scrape(&self, url: String) -> Result<(String, String), String> {
        eprintln!("spiders/svt: scraping {}", &url);
        let res = self.http_client
            .get(&url)
            .send()
            .await
            .expect("spiders/svt: downloading url");
        eprintln!("Status for {}: {}", &url, res.status());

        let body = res.text().await.expect("spiders/svt: read body");
        Ok((url, body))
    }

    async fn process(&self, url_item: (String, String)) -> Result<(), String> {
        eprintln!("spiders/svt: processing item for {}", url_item.0);
        let bare_path = url_item.0.as_str().trim_start_matches("http://"); 
        let bare_path = bare_path.trim_start_matches("https://"); 
        let path = self.out_path.join(bare_path);
        eprintln!("spiders/svt: output path for {}: {:?}", url_item.0, path);
        println!("{}", url_item.1);
        Ok(())
    }
}
