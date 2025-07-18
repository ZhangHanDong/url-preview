use super::is_twitter_url;
use crate::{Preview, PreviewError};
use scraper::{Html, Selector};
#[cfg(feature = "logging")]
use tracing::debug;

use crate::utils;

/// Metadata extractor, responsible for extracting preview information from webpage content
#[derive(Clone)]
pub struct MetadataExtractor;

impl Default for MetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract(&self, html: &str, url: &str) -> Result<Preview, PreviewError> {
        let document = Html::parse_document(html);
        if is_twitter_url(url) {
            if let Some(preview) = self.extract_twitter_metadata(&document, url) {
                return Ok(preview);
            }
        }
        // If not a Twitter URL or Twitter extraction failed, use generic extraction method
        self.extract_generic_metadata(&document, url)
    }

    fn extract_twitter_metadata(&self, document: &Html, url: &str) -> Option<Preview> {
        let selectors = [
            ("article[data-testid='tweet']", "Article selector"),
            ("div[data-testid='tweetText']", "Text selector"),
            ("div[data-testid='tweetPhoto'] img", "Image selector"),
            ("div[data-testid='videoPlayer']", "Video selector"),
            ("div[data-testid='User-Name']", "Username selector"),
        ];

        // Print matching results for all selectors
        for (selector_str, _desc) in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                #[cfg(feature = "logging")]
                {
                    let count = document.select(&selector).count();
                    debug!("{}: Found {} matches", _desc, count);
                }
                #[cfg(not(feature = "logging"))]
                {
                    let _count = document.select(&selector).count();
                }
            }
        }

        // Try to extract basic metadata
        let og_title = self.extract_title(document);
        let og_description = self.extract_description(document);
        let og_image = self.extract_image(document);

        #[cfg(feature = "logging")]
        {
            debug!("Basic metadata extraction results:");
            debug!("Title: {:?}", og_title);
            debug!("Description: {:?}", og_description);
            debug!("Image: {:?}", og_image);
        }

        // Return basic info even if specific tweet elements not found
        Some(Preview {
            url: url.to_string(),
            title: og_title,
            description: og_description,
            image_url: og_image,
            site_name: Some("X (formerly Twitter)".to_string()),
            favicon: Some("https://abs.twimg.com/favicons/twitter.ico".to_string()),
        })
    }

    fn extract_generic_metadata(
        &self,
        document: &Html,
        url: &str,
    ) -> Result<Preview, PreviewError> {
        let title = self.extract_title(document);
        let description = self.extract_description(document);
        let image_url = self.extract_image(document);
        let favicon = self.extract_favicon(document);
        let site_name = self.extract_site_name(document);

        let host = utils::pickup_host_from_url(url)?;

        let image_url = format_url(image_url, &host);

        let favicon = format_url(favicon, &host);

        Ok(Preview {
            url: url.to_string(),
            title,
            description,
            image_url,
            favicon,
            site_name,
        })
    }

    fn extract_title(&self, document: &Html) -> Option<String> {
        let og_title_selector = Selector::parse("meta[property='og:title']").ok()?;
        let title_selector = Selector::parse("title").ok()?;

        let og_title = document
            .select(&og_title_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

        // If there is no Open Graph title, try to get the regular title
        og_title
            .or_else(|| {
                document
                    .select(&title_selector)
                    .next()
                    .map(|el| el.inner_html())
            })
            .map(|s| s.trim().to_string())
    }

    fn extract_description(&self, document: &Html) -> Option<String> {
        let og_desc_selector = Selector::parse("meta[property='og:description']").ok()?;
        let meta_desc_selector = Selector::parse("meta[name='description']").ok()?;

        document
            .select(&og_desc_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .or_else(|| {
                document
                    .select(&meta_desc_selector)
                    .next()
                    .and_then(|el| el.value().attr("content"))
            })
            .map(|s| s.trim().to_string())
    }

    fn extract_image(&self, document: &Html) -> Option<String> {
        let og_image_selector =
            Selector::parse("meta[property='og:image'],meta[itemprop='image']").ok()?;

        document
            .select(&og_image_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
    }

    fn extract_favicon(&self, document: &Html) -> Option<String> {
        let favicon_selector =
            Selector::parse("link[rel='icon'], link[rel='shortcut icon']").ok()?;

        document
            .select(&favicon_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.trim().to_string())
    }

    fn extract_site_name(&self, document: &Html) -> Option<String> {
        let og_site_selector = Selector::parse("meta[property='og:site_name']").ok()?;

        document
            .select(&og_site_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
    }

    /// Create a preview from oEmbed data.
    ///
    /// Takes oEmbed HTML content as a string and extracts relevant metadata to create a preview.
    pub fn extract_from_oembed(&self, oembed: &str) -> Option<Preview> {
        let document = Html::parse_fragment(oembed);

        let text_selector = Selector::parse("p").ok()?;
        let link_selector = Selector::parse("a").ok()?;

        let tweet_text = document
            .select(&text_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        let image_link = document
            .select(&link_selector)
            .find(|a| {
                a.value()
                    .attr("href")
                    .map(|href| href.contains("t.co"))
                    .unwrap_or(false)
            })
            .and_then(|a| a.value().attr("href"))
            .map(String::from);

        let time = document
            .select(&link_selector)
            .next_back()
            .map(|el| el.text().collect::<String>());

        Some(Preview {
            url: String::new(),
            title: tweet_text.clone(),
            description: Some(format!(
                "{}{}",
                tweet_text.unwrap_or_default(),
                time.map(|t| format!(" (Posted: {t})"))
                    .unwrap_or_default()
            )),
            image_url: image_link,
            site_name: Some("X (formerly Twitter)".to_string()),
            favicon: Some("https://abs.twimg.com/favicons/twitter.ico".to_string()),
        })
    }
}

// Helper function to check if a URL is absolute and format it accordingly
fn format_url(url: Option<String>, host: &str) -> Option<String> {
    fn is_absolute_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    if let Some(url) = url {
        if is_absolute_url(&url) {
            Some(url)
        } else if url.starts_with('/') {
            Some(format!("{host}{url}"))
        } else {
            Some(format!("{host}/{url}"))
        }
    } else {
        None
    }
}
