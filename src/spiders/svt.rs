use crate::error::Error;

use async_trait::async_trait;
use regex::Regex;
use serde_json::Value as JsonValue;
use tokio::fs as tokio_fs;

use std::{
    fs,
    collections::HashMap,
    path::PathBuf,
    time::Duration,
};

mod domain;

pub use domain::{Content, Page, ArticleResponse};

pub struct SvtSpider {
    http_client: reqwest::Client,
    page_regex: Regex,
    topic_regex: Regex,
    out_path: PathBuf,
    crawled_data: HashMap<String, (String, i32, String)>,
}

impl SvtSpider {
    pub fn new(
        out_path: PathBuf,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .expect("spiders/svt: Building HTTP client");

        let page_regex = Regex::new(".*page=([0-9]*).*")
            .expect("spiders/svt: Compiling page regex");

        let topic_regex = Regex::new(".*/([a-z0-9]+)/.*")
            .expect("spiders/svt: Compiling topic regex");
        let crawled_data = HashMap::new();

        Self {
            http_client,
            page_regex,
            topic_regex,
            out_path,
            crawled_data,
        }
    }
}

const API_URL: &str = "https://api.svt.se/nss-api/page";
const ARTICLE_URL: &str = "https://api.svt.se/nss-api/page{}?q=articles";
const LIMIT: u32 = 50;

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

#[derive(Clone, Debug)]
pub struct SvtData {
    pub topic_name: String,
    pub json: JsonValue,
}

#[async_trait]
impl super::Spider for SvtSpider {
    type Item = SvtData;

    fn start_urls(&self) -> Vec<String> {
        let mut start_urls = Vec::new();
        for topic in TOPICS.iter() {
//             topic_name = topic
            let topic_name = if topic.contains('/') {
                topic.split('/').last().unwrap().to_string()
            } else {
                topic.clone()
            };
            let topic_url = format!("{}/{}?q=auto&limit={}&page=1", API_URL, topic, LIMIT);
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

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        log::debug!("spiders/svt: scraping {}", &url);
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        log::info!("Status for {}: {}", &url, response.status());

        let mut next_pages_links = Vec::new();
        let mut items = Vec::new();

        if url.contains("q=articles") {
            let article: ArticleResponse = response.json().await?;
            if article.articles.content.len() == 0 {
                return Err(Error::Internal(format!("No data found in {}", &url)));
            }
            if article.articles.content.len() > 1 {
                log::warn!("Found article with multiple content entries: {}", &url);
            }
            let captures = self.topic_regex.captures(&url).unwrap();
            let topic_name = captures.get(1).unwrap().as_str().to_string();
            for (i, content) in article.articles.content.into_iter().enumerate() {
                if i == 0 {
                    items.push(
                        SvtData {
                            topic_name: topic_name.clone(),
                            json: content,
                    });
                } else {
                    log::warn!("skipping multiple content: {:?}", content);
                }
            }

        } else {
            let page: Page = response.json().await?;

            for content in page.auto.content {
                if let Some(short_url) = content.url {
                    log::debug!("spiders/svt: short_url = {}", &short_url);
                    if self.crawled_data.contains_key(&short_url) {
                        log::debug!("  Article already saved, skipping remaining. Date: {:?}", content.published);
                        return Ok((items, next_pages_links));
                    }
                    let short_url_str = short_url.as_str().trim_start_matches("https://www.svt.se");
                    let new_url = format!("{}{}?q=articles", API_URL, short_url_str);
                    next_pages_links.push(new_url);
                }
            }
            let captures = self.page_regex.captures(&url).unwrap();
            let old_page_number = captures.get(1).unwrap().as_str().to_string();
            let mut new_page_number = old_page_number
                .parse::<usize>()
                .map_err(|_| Error::Internal("spiders/svt: parsing page number".to_string()))?;
            new_page_number += 1;
            let next_url = url.replace(
                format!("&page={}", old_page_number).as_str(),
                format!("&page={}", new_page_number).as_str(),
            );
            next_pages_links.push(next_url);
        }
        Ok((items, next_pages_links))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        log::info!("spiders/svt: processing item for {:?}", &item.topic_name);
        let article_id = item.json.get("id").ok_or_else(
            || {
                log::error!("missing id. data: {:?}", &item);
                Error::BadData("missing id".to_string())
            })?;
        log::debug!("article_id = {:?}", article_id);
        log::debug!("{:?}", item.json.get("published"));
        log::debug!("{:?}", item.json.get("modified"));
        let year: &str = if let Some(JsonValue::String(year)) = item.json.get("published") {
            &year[..4]
        } else if let Some(JsonValue::String(year)) = item.json.get("modified") {
            &year[..4]
        } else {
           "0"
        };
        log::debug!("year = {}", year);

        let year: &str = if year < "2004" {
            "nodate"
        } else if year > "2022" {
            log::warn!("found future date '{}'. article_id: {}", year, article_id);
            "nodate"
        } else {
            year
        };
        
        // let bare_path = url_item.0.as_str().trim_start_matches("http://");
        // let bare_path = bare_path.trim_start_matches("https://");
        let path = self.out_path
            .join(format!("svt-{}", year))
            .join(&item.topic_name);

        log::info!("creating dir: {:?}", &path);
        tokio_fs::create_dir_all(&path).await?;
        let path = path.join(format!("{}.json", article_id));
        let mut file = fs::File::create(path)?;
        serde_json::to_writer(file, &item.json)?;
        // eprintln!("spiders/svt: output path for {}: {:?}", url_item.0, path);
        // println!("{:?}", item);
        Ok(())
    }
}
