use crate::fetcher::FetchResult;
use crate::{Cache, Fetcher, MetadataExtractor, Preview, PreviewError, PreviewGenerator};
use async_trait::async_trait;
use url::Url;

#[derive(Clone, Default)]
pub enum CacheStrategy {
    #[default] UseCache,
    NoCache,
    ForceUpdate,
}

#[derive(Clone)]
pub struct UrlPreviewGenerator {
    pub cache: Cache,
    pub cache_strategy: CacheStrategy,
    pub fetcher: Fetcher,
    extractor: MetadataExtractor,
}

impl UrlPreviewGenerator {
    pub fn new(cache_capacity: usize, cache_strategy: CacheStrategy) -> Self {
        Self {
            cache: Cache::new(cache_capacity),
            cache_strategy,
            fetcher: Fetcher::new(),
            extractor: MetadataExtractor::new(),
        }
    }

    pub fn new_with_fetcher(
        cache_capacity: usize,
        cache_strategy: CacheStrategy,
        fetcher: Fetcher
    ) -> Self {
        Self {
            cache: Cache::new(cache_capacity),
            cache_strategy,
            fetcher,
            extractor: MetadataExtractor::new(),
        }
    }
}

// For Twitter url and Normal url
#[async_trait]
impl PreviewGenerator for UrlPreviewGenerator {
    async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        if let CacheStrategy::UseCache = self.cache_strategy {
            if let Some(cached) = self.cache.get(url).await {
                return Ok(cached);
            };
        };

        let _ = Url::parse(url)?;
        let content = self.fetcher.fetch(url).await?;

        let mut preview = match content {
            FetchResult::OEmbed(oembed) => self
                .extractor
                .extract_from_oembed(&oembed.html)
                .ok_or_else(|| {
                    PreviewError::ExtractError("Failed to extract from oEmbed".into())
                })?,
            FetchResult::Html(html) => self.extractor.extract(&html, url)?,
        };
        preview.url = url.to_string();
        if let CacheStrategy::UseCache = self.cache_strategy {
            self.cache.set(url.to_string(), preview.clone()).await;
        };
        Ok(preview)
    }
}
