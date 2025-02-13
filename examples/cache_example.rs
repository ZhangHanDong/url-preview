use std::error::Error;
use url_preview::{PreviewGenerator, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let generator = UrlPreviewGenerator::new(1000);
    // let url = "https://www.rust-lang.org";
    let url = "https://www.baidu.com";

    // 第一次获取 - 将从网络获取
    println!("First fetch (from network):");
    let start = std::time::Instant::now();
    let preview1 = generator.generate_preview(url).await?;
    println!("Time taken: {:?}", start.elapsed());
    println!("Title: {:?}\n", preview1.title);

    // 第二次获取 - 将从缓存获取
    println!("Second fetch (from cache):");
    let start = std::time::Instant::now();
    let preview2 = generator.generate_preview(url).await?;
    println!("Time taken: {:?}", start.elapsed());
    println!("Title: {:?}", preview2.title);

    Ok(())
}
