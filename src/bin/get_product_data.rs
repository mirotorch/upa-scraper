use reqwest::Error;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> () {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let mut product_infos: Vec<HashMap<String, String>> = Vec::new();

    let mut counter: i32 = 0;
    for line in reader.lines() {
        let url = line.expect("failed to read from stdin");
        let document = match read_page(&url).await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("failed to load HTML document: {}", e);
                continue;
            }
        };
        match read_data(&document) {
            Ok(doc) => product_infos.push(doc),
            Err(e) => eprintln!("failed to read product info: {}", e),
        }
    }
    // complete missing values
    let keys: HashSet<_> = product_infos
        .iter()
        .flat_map(|map| map.keys().cloned())
        .collect();

    for map in &mut product_infos {
        for key in &keys {
            map.entry(key.clone()).or_insert_with(|| "NaN".to_string());
        }
    }

    // let mut tsv = match File::create("data.tsv") {
    //     Ok(f) => f,
    //     Err(e) => panic!("couldn't create data.tsv file: {}", e),
    // };
}

async fn read_page(url: &str) -> Result<Html, Error> {
    let body = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&body);
    return Ok(document);
}

// selectors for read_data
static details_selector: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div#product-details").unwrap());
static title_selector: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1.product-title").unwrap());
static th_selector: LazyLock<Selector> = LazyLock::new(|| Selector::parse("th").unwrap());
static tr_selector: LazyLock<Selector> = LazyLock::new(|| Selector::parse("tr").unwrap());
static td_selector: LazyLock<Selector> = LazyLock::new(|| Selector::parse("td").unwrap());

fn read_data(document: &Html) -> Result<HashMap<String, String>, &'static str> {
    let mut data = HashMap::new();

    // get product name
    let title = match document.select(&title_selector).next() {
        Some(x) => x,
        None => return Err("title not found"),
    };
    data.insert("Name".to_string(), title.text().collect::<String>());

    let product_details = match document.select(&details_selector).next() {
        Some(x) => x,
        None => return Err("product details not found"),
    };

    for tr in product_details.select(&tr_selector) {
        let th = tr.select(&th_selector).next();
        let td = tr.select(&td_selector).next();
        if th.is_some() && td.is_some() {
            data.insert(
                th.unwrap().text().collect::<String>(),
                td.unwrap().text().collect::<String>(),
            );
        }
    }

    return Ok(data);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_data() {
        if cfg!(debug_assertions) {
            let test_page = match fs::read_to_string("test_page.html") {
                Err(e) => panic!("couldn't open test page: {}", e),
                Ok(f) => f,
            };
            let document = Html::parse_document(&test_page);
            let data = match read_data(&document) {
                Ok(x) => x,
                Err(e) => panic!("error reading product info: {}", e),
            };
            println!("{:#?}", data);
        }
    }
}
