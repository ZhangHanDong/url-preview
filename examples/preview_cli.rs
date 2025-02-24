use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use tokio::time::{sleep, Duration};
use url_preview::{PreviewGenerator, UrlPreviewGenerator, CacheStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "URL Preview Generator Example".bold().green());
    println!("{}", "================================".green());

    let generator = UrlPreviewGenerator::new(1000, CacheStrategy::UseCache);

    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com",
        "https://news.ycombinator.com",
        "https://www.wikipedia.org",
    ];

    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    for url in urls {
        match generator.generate_preview(url).await {
            Ok(preview) => {
                pb.inc(1);

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

                println!();
            }
            Err(e) => {
                pb.inc(1);
                eprintln!("{}: {} - {}", "Error".bold().red(), url, e);
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    pb.finish_with_message("All URLs processed!");

    Ok(())
}
