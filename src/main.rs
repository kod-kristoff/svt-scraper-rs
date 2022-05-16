use svt_scraper::{spiders, Crawler};

use clap::{Arg, Command};

#[tokio::main]
async fn main() {
    // Parse command line args, print help if none are given
    let args = parse_args();
    eprintln!("args = {:?}", args);

    match args.command {
        Cmd::Crawl { retry, force, debug } => {
            let crawler = Crawler::new();
            if retry {
                println!("\nTrying to crawl pages that failed last time ...");
                if force {
                    println!("Argument '--force' is ignored when recrawling failed pages.");
                }
                let spider = Arc::new(spiders::svt::SvtSpider::new(true));
                crawler.run(spider).await;
            } else {
                println!("\nStarting to crawl svt.se ...");
//             time.sleep(5)
                let spider = Arc::new(spiders::svt::SvtSpider::new(debug));
                crawler.run(spider).await;
            }
        },
        Cmd::Summary => {
            println!("\nCalculating summary of collected articles ...");
//         SvtParser().get_articles_summary()
        },
        Cmd::Xml { r#override } => {
            println!("\nPreparing to convert articles to XML ...");
//         process_articles(override_existing=args.override)
        },
        Cmd::BuildIndex { out } => {
            println!("\nBuilding an index of crawled files based on the downloaded JSON files ...");
//         crawled_data_from_files(args.out)
        }
// 
// 
//     ## DEBUG STUFF
// 
//     # SvtParser().get_article("/nyheter/inrikes/toppmote-om-arktis-i-kiruna", "inrikes")
// 
//     # with open("data/svt-2020/konsument/28334881.json") as f:
//     #     article_json = json.load(f)
//     #     xml = process_article(article_json[0])
//     #     print(xml)
    }
}

// -------------------------------------------------------------------------------
//  Define the command line args
// -------------------------------------------------------------------------------
fn parse_args() -> Args {
    let matches = Command::new("svt-crawler")
        .about("Programme for crawling svt.se for news articles and converting the data to XML.")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("crawl")
                .about("Crawl svt.se and download news articles")
                .arg(
                    Arg::new("retry")
                        .short('r')
                        .long("retry") 
                        .help("try to crawl pages that have failed previously")
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .help("crawl all pages even if they have been crawled before")
                )
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .long("debug")
                        .help("print some debug info while crawling")
                )
        )
        .subcommand(
            Command::new("summary")
                .about("Print summary of collected data")
        )
        .subcommand(
            Command::new("xml")
                .about("Convert articles from JSON to XML")
                .arg(
                    Arg::new("override")
                        .short('o')
                        .long("override")
                        .help("override existing xml files")
                )
        )
        .subcommand(
            Command::new("build-index")
                .about("Compile an index of the crawled data based on the downloaded files")
                .arg(
                    Arg::new("out")
                        .long("out")
                        .takes_value(true)
                        .value_name("OUT")
                        .default_value("crawled_pages_from_files.json")
                        .help(&*format!("name of the output file (will be stored in '{}')", DATADIR))
                )
        )
        .get_matches();
    let command = match matches.subcommand() {
        Some(("crawl", sub_m)) => {
            Cmd::Crawl {
                force: sub_m.is_present("force"),
                retry: sub_m.is_present("retry"),
                debug: sub_m.is_present("debug"),
            }
        },
        Some(("summary", _)) => Cmd::Summary,
        Some(("xml", sub_m)) => {
            Cmd::Xml {
                r#override: sub_m.is_present("override"),
            }
        },
        Some(("build-index", sub_m)) => {
            let mut out = PathBuf::from(DATADIR);
            out.push(sub_m.value_of("out").unwrap());
            Cmd::BuildIndex { out }
        },
        _ => { unreachable!() }
    };
    Args { command } 
}

#[derive(Debug)]
struct Args {
    command: Cmd,
}

#[derive(Debug)]
enum Cmd {
    Crawl {
        retry: bool,
        force: bool,
        debug: bool,
    },
    Summary,
    Xml {
        r#override: bool,
    },
    BuildIndex {
        out: PathBuf,
    },
}
