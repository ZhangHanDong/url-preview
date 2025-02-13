use std::error::Error;
use url_preview::{PreviewGenerator, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let generator = UrlPreviewGenerator::new(1000);

    // 创建多个 URL 的异步任务
    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com",
        "https://news.ycombinator.com",
    ];

    // 使用 tokio::spawn 创建多个异步任务
    let handles: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let generator = generator.clone();
            let url = url.to_string();

            tokio::spawn(async move {
                match generator.generate_preview(&url).await {
                    Ok(preview) => (url, Ok(preview)),
                    Err(e) => (url, Err(e)),
                }
            })
        })
        .collect();

    // 等待所有任务完成
    for handle in handles {
        let (url, result) = handle.await?;
        match result {
            Ok(preview) => {
                println!("Successfully fetched preview for {}", url);
                println!("Title: {:?}", preview.title);
            }
            Err(e) => {
                println!("Failed to fetch preview for {}: {}", url, e);
            }
        }
    }

    Ok(())
}
