use chrono::{DateTime, FixedOffset};
use clap::Parser;
use log::{LevelFilter, info};
use scraper::*;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::path::PathBuf;

const VERKEHRSMELDUNG_URL: &'static str =
    "https://www.holding-graz.at/de/category/verkehrsmeldungen/";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path for the json database file
    #[arg(short, long, default_value = "graz-opnv-bot.json")]
    database: PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Announcement {
    content: String,
    link: String,
    datetime: DateTime<FixedOffset>,
}

impl PartialEq for Announcement {
    fn eq(&self, other: &Self) -> bool {
        self.datetime == other.datetime
    }
}

fn fetch_online_announcements() -> Vec<Announcement> {
    let mut announcements = Vec::new();
    let resp = reqwest::blocking::get(VERKEHRSMELDUNG_URL);

    if let Ok(body) = resp {
        if body.status().is_success() {
            if let Ok(content) = body.text() {
                let document = Html::parse_document(&content);
                let selector = Selector::parse(r#"div[class="related-teaser__content"]"#).unwrap();

                let mut count = 0;

                for announcement in document.select(&selector) {
                    count += 1;
                    let mut new_announcement = Announcement::default();

                    let link_selector = Selector::parse(r#"a"#).unwrap();
                    for link in announcement.select(&link_selector).skip(1) {
                        if let Some(l) = link.attr("href") {
                            new_announcement.link = l.to_string();
                        }
                    }

                    let time_selector = Selector::parse(r#"time"#).unwrap();
                    for time in announcement.select(&time_selector) {
                        if let Some(t) = time.attr("datetime") {
                            new_announcement.datetime = DateTime::parse_from_rfc3339(t).unwrap();
                        }
                    }

                    let text_selector = Selector::parse(r#"p"#).unwrap();
                    for text in announcement.select(&text_selector) {
                        let mut content = String::new();
                        html_escape::decode_html_entities_to_string(
                            text.inner_html().trim(),
                            &mut content,
                        );

                        new_announcement.content = content;
                    }

                    info!("Fetched announcement: {:?}", new_announcement);
                    announcements.push(new_announcement);
                }
                info!("Fetched {} announcements", count);
            } else {
                panic!("Body did not contain text");
            }
        } else {
            panic!("Response was bad: {}", body.status());
        }
    } else {
        panic!("Request failed: {}", resp.err().unwrap());
    }

    announcements.sort_by_key(|a| a.datetime);
    announcements
}

fn create_disk_database(database: &PathBuf, announcements: &Vec<Announcement>) {
    let new_db = File::create(database);
    if let Ok(db) = new_db {
        serde_json::to_writer_pretty(db, announcements).unwrap();
    } else {
        panic!("Could not create new database: {}", new_db.err().unwrap());
    }
}

fn load_disk_announcements(database: &PathBuf) -> Vec<Announcement> {
    let check = std::fs::exists(database);
    if let Ok(exists) = check {
        if !exists {
            info!("Database does not exist, creating...");
            let online_announcements = fetch_online_announcements();
            create_disk_database(database, &online_announcements);
        }
    } else {
        panic!(
            "Failed to check if database exists: {}",
            check.err().unwrap()
        );
    }

    let db = File::open(database);

    if let Ok(db) = db {
        let raw_announcements: Result<Vec<Announcement>, serde_json::Error> =
            serde_json::from_reader(db);
        if let Ok(mut announcements) = raw_announcements {
            announcements.sort_by_key(|a| a.datetime);
            announcements
        } else {
            panic!(
                "Failed to parse database: {}",
                raw_announcements.err().unwrap()
            );
        }
    } else {
        panic!("Failed to open database: {}", db.err().unwrap());
    }
}

fn main() {
    let args = Cli::parse();

    if args.verbose {
        SimpleLogger::new()
            .with_level(LevelFilter::Trace)
            .init()
            .unwrap();
    } else {
        SimpleLogger::new()
            .with_level(LevelFilter::Warn)
            .init()
            .unwrap();
    }

    info!("Using database `{}`", args.database.display());

    let mut disk_announcments = load_disk_announcements(&args.database);
    let mut online_announcements = fetch_online_announcements();

    let mut new_found = false;

    online_announcements
        .iter()
        .filter(|a| !disk_announcments.contains(&a))
        .for_each(|a| {
            println!("Neue Verkehrsmeldung 🚉🚍\n{}\nvom {}\nMehr Infos: {}\n\nAutomatisiert durch OPNV-Graz Bot 🤖 ()", a.content, a.datetime.format("%F %T"), a.link);
            new_found = true;
        });

    if !new_found {
        std::process::exit(2);
    }

    disk_announcments.append(&mut online_announcements);
    disk_announcments.sort_by_key(|a| a.datetime);
    disk_announcments.dedup_by(|a, b| a.datetime == b.datetime);

    create_disk_database(&args.database, &disk_announcments);
}
