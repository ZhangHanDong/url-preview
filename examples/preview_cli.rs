use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use tokio::time::{sleep, Duration};
use url_preview::{PreviewGenerator, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "URL Preview Generator Example".bold().green());
    println!("{}", "================================".green());

    // 创建一个预览生成器实例，设置缓存大小为 1000 条
    let generator = UrlPreviewGenerator::new(1000);

    // 准备一些示例 URL
    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com",
        "https://news.ycombinator.com",
        "https://www.wikipedia.org",
    ];

    // 创建进度条
    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    // 为每个 URL 生成预览
    for url in urls {
        // 生成预览
        match generator.generate_preview(url).await {
            Ok(preview) => {
                // 清理进度条显示
                pb.inc(1);

                // 打印预览信息
                println!("\n{}", "URL Preview".bold().blue());
                println!("{}", "---------------".blue());
                println!("{}: {}", "URL".bold(), url);

                if let Some(title) = preview.title {
                    println!("{}: {}", "Title".bold(), title);
                }

                if let Some(description) = preview.description {
                    println!(
                        "{}: {}",
                        "Description".bold(),
                        // 如果描述太长，截断它
                        if description.len() > 100 {
                            format!("{}...", &description[..100])
                        } else {
                            description
                        }
                    );
                }

                if let Some(image) = preview.image_url {
                    println!("{}: {}", "Image".bold(), image);
                }

                if let Some(site_name) = preview.site_name {
                    println!("{}: {}", "Site Name".bold(), site_name);
                }

                println!(); // 空行分隔
            }
            Err(e) => {
                pb.inc(1);
                eprintln!("{}: {} - {}", "Error".bold().red(), url, e);
            }
        }

        // 在请求之间添加小延迟，避免太快发送请求
        sleep(Duration::from_millis(500)).await;
    }

    pb.finish_with_message("All URLs processed!");

    Ok(())
}
