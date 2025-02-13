use std::error::Error;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use url_preview::{setup_logging, Fetcher, LogConfig, PreviewGenerator, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志系统
    log_initialize();

    info!("Starting Twitter/X Preview Example");

    // // 创建 Twitter 专用客户端
    let generator = UrlPreviewGenerator::new_with_fetcher(1000, Fetcher::new_twitter_client());

    let url = "https://x.com/blackanger/status/1888945450650362251";
    info!(url = %url, "Generating preview for URL");

    match generator.generate_preview(url).await {
        Ok(preview) => {
            info!("Preview generated successfully");

            // 使用结构化日志记录预览信息
            info!(
                url = %preview.url,
                title = %preview.title.as_deref().unwrap_or("N/A"),
                description = %preview.description.as_deref().unwrap_or("N/A"),
                image = %preview.image_url.as_deref().unwrap_or("N/A"),
                site = %preview.site_name.as_deref().unwrap_or("N/A"),
                "Preview details"
            );

            // 对每个字段进行详细的日志记录
            if let Some(title) = &preview.title {
                debug!(title = %title, "Preview title found");
            } else {
                warn!("No title found in preview");
            }

            if let Some(desc) = &preview.description {
                debug!(description = %desc, "Preview description found");
            } else {
                warn!("No description found in preview");
            }

            if let Some(image) = &preview.image_url {
                debug!(image_url = %image, "Preview image found");
            } else {
                warn!("No image found in preview");
            }

            if let Some(site) = &preview.site_name {
                debug!(site_name = %site, "Site name found");
            } else {
                warn!("No site name found in preview");
            }
        }
        Err(e) => {
            // 错误处理使用结构化日志
            error!(error = %e, "Failed to generate preview");

            if let Some(source) = e.source() {
                error!(
                    error = %e,
                    source = %source,
                    "Detailed error information"
                );
            }

            // 记录故障排查建议
            warn!("Please check the following:");
            warn!("1. URL accessibility");
            warn!("2. Network connectivity");
            warn!("3. Authentication requirements");
        }
    }

    info!("Preview generation process completed");
    Ok(())
}

pub fn log_initialize() {
    let log_config = LogConfig {
        log_dir: PathBuf::from("logs"),
        log_level: "info".into(),
        console_output: true,
        file_output: true,
    };

    setup_logging(log_config);
    info!("URL Preview system initialized with logging configuration");
}
