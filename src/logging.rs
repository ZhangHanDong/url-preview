use crate::utils::truncate_str;
use crate::Preview;
use std::fmt::Display;
use std::path::PathBuf;
use tracing::{debug, error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt as subscriber_fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

#[derive(Debug)]
pub struct LogConfig {
    pub log_dir: PathBuf,
    pub log_level: String,
    pub console_output: bool,
    pub file_output: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: "logs".into(),
            log_level: "info".into(),
            console_output: true,
            file_output: true,
        }
    }
}

fn create_separator(width: usize, ch: char) -> String {
    std::iter::repeat_n(ch, width).collect()
}

pub fn log_preview_card(preview: &Preview, url: &str) {
    const CARD_WIDTH: usize = 80;
    const CONTENT_WIDTH: usize = CARD_WIDTH - 2;

    fn wrap_text(text: &str, width: usize) -> String {
        let mut wrapped = String::new();
        let mut line_length = 0;

        for word in text.split_whitespace() {
            if line_length + word.len() + 1 > width {
                wrapped.push('\n');
                wrapped.push_str("  ");
                wrapped.push_str(word);
                line_length = word.len() + 2;
            } else {
                if line_length > 0 {
                    wrapped.push(' ');
                    line_length += 1;
                }
                wrapped.push_str(word);
                line_length += word.len();
            }
        }
        wrapped
    }

    let url_wrapped = wrap_text(url, CONTENT_WIDTH - 5);
    let title_wrapped = wrap_text(preview.title.as_deref().unwrap_or("N/A"), CONTENT_WIDTH - 7);
    let desc_wrapped = wrap_text(
        preview.description.as_deref().unwrap_or("N/A"),
        CONTENT_WIDTH - 6,
    );
    let image_wrapped = wrap_text(
        preview.image_url.as_deref().unwrap_or("N/A"),
        CONTENT_WIDTH - 7,
    );
    let site_wrapped = wrap_text(
        preview.site_name.as_deref().unwrap_or("N/A"),
        CONTENT_WIDTH - 6,
    );

    let horizontal_line = "═".repeat(CARD_WIDTH - 2);

    info!(
        "\n╔{}╗\n\
         URL: {}\n\
         Title: {}\n\
         Desc: {}\n\
         Image: {}\n\
         Site: {}\n\
         ╚{}╝",
        horizontal_line,
        url_wrapped,
        title_wrapped,
        desc_wrapped,
        image_wrapped,
        site_wrapped,
        horizontal_line,
    );
}

pub fn log_error_card<E: Display + std::error::Error>(url: &str, error: &E) {
    const CARD_WIDTH: usize = 70;
    const CONTENT_WIDTH: usize = CARD_WIDTH - 8;

    let top_bottom = create_separator(CARD_WIDTH - 2, '═');
    let middle = create_separator(CARD_WIDTH - 2, '─');

    let mut error_details = error.to_string();
    if let Some(source) = error.source() {
        error_details = format!("{error_details} (原因: {source})");
    }

    error!(
        "\n╔═{}═╗\n\
         ║ URL: {:<width$} ║\n\
         ║{}║\n\
         ║ 错误: {:<width$} ║\n\
         ╚═{}═╝",
        top_bottom,
        truncate_str(url, CONTENT_WIDTH),
        middle,
        truncate_str(&error_details, CONTENT_WIDTH),
        top_bottom,
        width = CONTENT_WIDTH
    );
}

pub fn setup_logging(config: LogConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let mut layers = Vec::new();

    if config.console_output {
        let console_layer = subscriber_fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .with_span_events(subscriber_fmt::format::FmtSpan::FULL)
            .pretty();
        layers.push(console_layer.boxed());
    }

    if config.file_output {
        std::fs::create_dir_all(&config.log_dir).expect("Failed to create log directory");

        let file_appender =
            RollingFileAppender::new(Rotation::DAILY, &config.log_dir, "url-preview.log");

        let file_layer = subscriber_fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .with_writer(file_appender);

        layers.push(file_layer.boxed());
    }

    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .try_init()
        .expect("Failed to set global default subscriber");

    debug!("Logging system initialized with config: {:?}", config);
}

pub struct LogLevelGuard {
    _guard: tracing::dispatcher::DefaultGuard,
}

impl LogLevelGuard {
    pub fn set_level(level: &str) -> Self {
        let filter = EnvFilter::new(level);
        let subscriber = tracing_subscriber::registry()
            .with(subscriber_fmt::layer())
            .with(filter);

        LogLevelGuard {
            _guard: tracing::subscriber::set_default(subscriber),
        }
    }
}
