use reqwest::Error;
use scraper::{Html, Selector};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

async fn read_data(url: &str) -> Result<HashMap<String, String>, Error> {
    let body = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&body);

    let mut data = HashMap::new();
    let mut selector = Selector::parse("h1.product-title").unwrap();
    data.insert(
        "Name".to_string(),
        document.select(&selector).next().unwrap().text().collect(),
    );

    selector = Selector::parse("div#product-details").unwrap();

    let product_details = match document.select(&selector).next() {
        Some(x) => x,
        None => panic!("product details not found"),
    };

    selector = Selector::parse("tr").unwrap();

    for tr in product_details.select(&selector) {
        let mut children = tr.children().filter_map(|c| c.value().as_element());
        let th = tr
            .select(&Selector::parse("th").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        let td = tr
            .select(&Selector::parse("td").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        data.insert(th, td);
    }

    return Ok(data);
}
