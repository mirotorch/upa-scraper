use reqwest::Error;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use std::sync::LazyLock;

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let mut product_infos: Vec<HashMap<String, String>> = Vec::new();

    for line in reader.lines() {
        let url = line.expect("failed to read from stdin") + "?recaptcha=pass";
        // println!("Reading {}", url);
        let document = match read_page(&url).await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("failed to load HTML document: {}", e);
                continue;
            }
        };
        match read_data(&document) {
            Ok(doc) => {
                product_infos.push(doc);
                // println!("Ok");
            }
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

    let mut header: Vec<String> = Vec::new();
    header.push("Name".to_string());
    header.extend(keys.iter().filter(|x| *x != "Name").cloned());
    println!("{}", header.join("\t"));

    for map in product_infos {
        let line = header
            .iter()
            .map(|att| map.get(att).map(|val| val.as_str()).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\t");

        println!("{}", line);
    }
    // println!("Product data have been saved into data.tsv");
}

async fn read_page(url: &str) -> Result<Html, Error> {
    let client = reqwest::Client::new();
    let body = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0",
        )
        .send()
        .await?
        .text()
        .await?;
    let document = Html::parse_document(&body);
    return Ok(document);
}

// selectors for read_data
static DETAILS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div#product-details").unwrap());
static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1.product-title").unwrap());
static TR_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("tr").unwrap());
static TH_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("th").unwrap());
static TD_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("td").unwrap());

fn read_data(document: &Html) -> Result<HashMap<String, String>, &'static str> {
    let mut data = HashMap::new();

    // get product name
    let title = match document.select(&TITLE_SELECTOR).next() {
        Some(x) => x,
        None => return Err("title not found"),
    };
    data.insert("Name".to_string(), title.text().collect::<String>());

    let product_details = match document.select(&DETAILS_SELECTOR).next() {
        Some(x) => x,
        None => return Err("product details not found"),
    };

    for tr in product_details.select(&TR_SELECTOR) {
        let th = tr.select(&TH_SELECTOR).next();
        let td = tr.select(&TD_SELECTOR).next();
        if th.is_some() && td.is_some() {
            data.insert(
                th.unwrap().text().collect::<String>(),
                td.unwrap().text().collect::<String>().replace("\n", ";"),
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
