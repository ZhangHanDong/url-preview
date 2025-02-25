#[cfg(test)]
mod tests {
    use url_preview::{CacheStrategy, PreviewGenerator, PreviewService, UrlPreviewGenerator};

    #[tokio::test]
    async fn test_preview_generator() {
        let generator = UrlPreviewGenerator::new(100, CacheStrategy::UseCache);

        let preview = generator
            .generate_preview("https://www.rust-lang.org")
            .await
            .unwrap();

        assert!(preview.title.is_some());
        assert!(preview.url.contains("rust-lang.org"));

        let result = generator.generate_preview("not-a-valid-url").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let generator = UrlPreviewGenerator::new(100, CacheStrategy::UseCache);
        let url = "https://www.rust-lang.org";

        let first_fetch = std::time::Instant::now();
        let _ = generator.generate_preview(url).await.unwrap();
        let first_duration = first_fetch.elapsed();

        let second_fetch = std::time::Instant::now();
        let _ = generator.generate_preview(url).await.unwrap();
        let second_duration = second_fetch.elapsed();

        assert!(second_duration < first_duration);
    }

    #[tokio::test]
    async fn test_no_cache() {
        let preview_service = PreviewService::with_no_cache();
        let url_list = vec![
            "https://www.rust-lang.org",
            "https://github.com/ZhangHanDong/url-preview",
            "https://x.com/VicentYip/status/1893861564760887571",
        ];

        for url in url_list {
            let _ = preview_service.generate_preview(url).await.unwrap();
            let cache = preview_service.default_generator.cache.get(url).await;
            assert!(cache.is_none());
        }
    }
}
