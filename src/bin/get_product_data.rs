use once_cell::sync::Lazy;
use reqwest::Error;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let mut product_infos: Vec<HashMap<String, String>> = Vec::new();
    let header = vec![
        "Name",
        "Price",
        "Brand",
        "Response Time",
        "Refresh Rate",
        "Resolution",
        "Panel",
    ];
    let attributes: HashSet<String> = header.iter().copied().map(|x| x.to_string()).collect();

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
        match read_data(&document, &attributes) {
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

    println!("{}", header.join("\t"));

    for map in product_infos {
        let line = header
            .iter()
            .map(|att| map.get(*att).map(|val| val.as_str()).unwrap_or("NaN"))
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
static TITLE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("h1.product-title").unwrap());
static PRICE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.price-current").unwrap());
static DETAILS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div#product-details").unwrap());
static TR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr").unwrap());
static TH_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("th").unwrap());
static TD_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("td").unwrap());

fn read_data(
    document: &Html,
    attributes: &HashSet<String>,
) -> Result<HashMap<String, String>, &'static str> {
    let mut data = HashMap::new();

    // get product name
    let title = match document.select(&TITLE_SELECTOR).next() {
        Some(x) => x,
        None => return Err("title not found"),
    };
    data.insert("Name".to_string(), title.text().collect::<String>());

    // get current price
    let price = match document.select(&PRICE_SELECTOR).next() {
        Some(x) => x,
        None => return Err("price not found"),
    };
    data.insert("Price".to_string(), price.text().collect::<String>());

    let product_details = match document.select(&DETAILS_SELECTOR).next() {
        Some(x) => x,
        None => return Err("product details not found"),
    };

    for tr in product_details.select(&TR_SELECTOR) {
        let th = tr.select(&TH_SELECTOR).next();
        let td = tr.select(&TD_SELECTOR).next();
        if th.is_some() && td.is_some() {
            let th_text = th.unwrap().text().collect::<String>().trim().to_string();
            if attributes.contains(&th_text) {
                data.insert(
                    th_text,
                    td.unwrap()
                        .text()
                        .collect::<String>()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
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
            let header = vec![
                "Name",
                "Price",
                "Brand",
                "Response Time",
                "Refresh Rate",
                "Resolution",
                "Panel",
            ];
            let attributes: HashSet<String> =
                header.iter().copied().map(|x| x.to_string()).collect();
            let data = match read_data(&document, &attributes) {
                Ok(x) => x,
                Err(e) => panic!("error reading product info: {}", e),
            };
            println!("{:#?}", data);
        }
    }
}
