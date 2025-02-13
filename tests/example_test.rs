#[cfg(test)]
mod tests {
    use url_preview::{PreviewGenerator, UrlPreviewGenerator};

    #[tokio::test]
    async fn test_preview_generator() {
        let generator = UrlPreviewGenerator::new(100);

        // 测试有效的 URL
        let preview = generator
            .generate_preview("https://www.rust-lang.org")
            .await
            .unwrap();

        assert!(preview.title.is_some());
        assert!(preview.url.contains("rust-lang.org"));

        // 测试无效的 URL
        let result = generator.generate_preview("not-a-valid-url").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let generator = UrlPreviewGenerator::new(100);
        let url = "https://www.rust-lang.org";

        // 第一次获取
        let first_fetch = std::time::Instant::now();
        let _ = generator.generate_preview(url).await.unwrap();
        let first_duration = first_fetch.elapsed();

        // 第二次获取（应该从缓存中获取）
        let second_fetch = std::time::Instant::now();
        let _ = generator.generate_preview(url).await.unwrap();
        let second_duration = second_fetch.elapsed();

        // 第二次获取应该明显快于第一次
        assert!(second_duration < first_duration);
    }
}
