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

const API_URL: &str = "https://api.svt.se/nss-api/page/";
const ARTICLE_URL: &str = "https://api.svt.se/nss-api/page{}?q=articles";
const LIMIT: u32 = 50;
const LIMIT_STR: &str = "50";

lazy_static! {
    static ref LOCAL: Vec<&'static str> = vec![
       "blekinge",
       "dalarna",
       "gavleborg",
       "halland",
       "helsingborg",
       "jamtland",
       "jonkoping",
       "norrbotten",
       "skane",
       "smaland",
       "stockholm",
       "sodertalje",
       "sormland",
       "uppsala",
       "varmland",
       "vast",
       "vasterbotten",
       "vasternorrland",
       "vastmanland",
       "orebro",
       "ost",
    ];
    static ref TOPICS: Vec<String> = {
        let mut topics: Vec<String> = vec![
            String::from("nyheter/ekonomi"),
            String::from("nyheter/granskning"),
            String::from("nyheter/inrikes"),
            String::from("nyheter/svtforum"),
            String::from("nyheter/nyhetstecken"),
            String::from("nyheter/vetenskap"),
            String::from("nyheter/konsument"),
            String::from("nyheter/utrikes"),
            String::from("sport"),
            String::from("vader"),
            String::from("kultur"),
        ];
        for area in LOCAL.iter() {
            topics.push(format!("nyheter/lokalt/{}", area));
        }
        topics
    };
}

#[async_trait]
impl super::Spider for SvtSpider {
    fn start_urls(&self) -> Vec<String> {
        let mut start_urls = Vec::new();
        for topic in TOPICS.iter() {
//             topic_name = topic
            let topic_name = if topic.contains('/') {
                topic.clone()
            } else {
                topic.split('/').last().unwrap().to_string()
            };
            let topic_url = format!("{}{}/", API_URL, topic);
//             let response = self.http_client
//                 .get(&topic_url)
//                 .query(&[("q", "auto"), ("limit", LIMIT_STR), ("page", "1")])
//                 .send()
//                 .expect("svt_parser: get first page");
//             let firstpage: Page = response.json().expect("crawl: deserialize");
// //             items = firstpage.get("auto", {}).get("pagination", {}).get("totalAvailableItems", 0)
//             let pages = firstpage.auto.pagination.total_available_items / LIMIT;
//             println!(
//                 "\nCrawling {}: {} items, {} pages",
//                 topic,
//                 firstpage.auto.pagination.total_available_items,
//                 pages,
//             );
            start_urls.push(topic_url);
            // self.get_urls(topic_name, topic_url, pages, firstpage, force)
        }
        start_urls
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
        // let path = self.out_path.join(bare_path);
        let path = bare_path;
        eprintln!("spiders/svt: output path for {}: {:?}", url_item.0, path);
        println!("{}", url_item.1);
        Ok(())
    }
}
