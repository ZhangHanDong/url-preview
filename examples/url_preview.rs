use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use url_preview::{log_error_card, log_preview_card, setup_logging, LogConfig, PreviewService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    log_initialize();

    // 创建预览服务
    let preview_service = PreviewService::new(1);

    // 处理任何 URL，服务会自动选择合适的处理器
    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com/zed-industries/zed",
        "https://news.ycombinator.com",
        "https://www.wikipedia.org",
        "https://x.com/blackanger/status/1888945450650362251",
    ];

    for url in urls {
        match preview_service.generate_preview(url).await {
            Ok(preview) => {
                log_preview_card(&preview, url);
            }
            Err(e) => {
                log_error_card(url, &e);
            }
        }
    }

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
