use reqwest::Error;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let urls = [
        "https://www.newegg.com/LCD-LED-Monitor/SubCategory/ID-20/Page-1?PageSize=96",
        "https://www.newegg.com/LCD-LED-Monitor/SubCategory/ID-20/Page-2?PageSize=96",
    ];

    let handles: Vec<_> = urls
        .iter()
        .map(|url| tokio::spawn(write_urls(url)))
        .collect();

    for handle in handles {
        match handle.await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error executing task: {}", e)
            }
        }
    }

    Ok(())
}

async fn write_urls(url: &str) -> Result<(), Error> {
    let body = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("a.item-title").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            println!("{}", href);
        }
    }
    Ok(())
}
