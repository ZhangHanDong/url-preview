use std::error::Error;
use url_preview::{PreviewGenerator, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let generator = UrlPreviewGenerator::new(1000);
    // let url = "https://www.rust-lang.org";
    let url = "https://www.baidu.com";

    println!("First fetch (from network):");
    let start = std::time::Instant::now();
    let preview1 = generator.generate_preview(url).await?;
    println!("Time taken: {:?}", start.elapsed());
    println!("Title: {:?}\n", preview1.title);

    println!("Second fetch (from cache):");
    let start = std::time::Instant::now();
    let preview2 = generator.generate_preview(url).await?;
    println!("Time taken: {:?}", start.elapsed());
    println!("Title: {:?}", preview2.title);

    Ok(())
}
