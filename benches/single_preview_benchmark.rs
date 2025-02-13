use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use url_preview::{Cache, Preview, PreviewService};

const MOCK_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <meta property="og:title" content="Test Title">
    <meta property="og:description" content="Test Description">
    <meta property="og:image" content="https://example.com/image.jpg">
    <link rel="icon" href="https://example.com/favicon.ico">
</head>
<body>
    <h1>Test Content</h1>
</body>
</html>"#;

fn bench_single_preview(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let preview_service = PreviewService::new();

    let mut group = c.benchmark_group("url_preview");

    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    rt.block_on(async {
        let url = "https://www.rust-lang.org";
        let _ = preview_service.generate_preview(url).await;
    });

    group.bench_function("cached_preview", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                preview_service
                    .generate_preview("https://www.rust-lang.org")
                    .await
                    .unwrap(),
            )
        });
    });

    let counter = Arc::new(AtomicUsize::new(0));

    group.bench_function("uncached_preview", |b| {
        let counter = Arc::clone(&counter);
        b.to_async(&rt).iter(|| async {
            let current = counter.fetch_add(1, Ordering::SeqCst);
            let unique_url = format!("https://example.com/page{}", current);
            black_box(preview_service.generate_preview(&unique_url).await.unwrap())
        });
    });

    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_operations");

    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    let cache = Cache::new(1000);
    let test_preview = generate_test_preview();

    let counter = Arc::new(AtomicUsize::new(0));

    group.bench_function("cache_write", |b| {
        let counter = Arc::clone(&counter);
        let test_preview = test_preview.clone();
        b.to_async(&rt).iter(|| async {
            let current = counter.fetch_add(1, Ordering::SeqCst);
            let key = format!("key{}", current);
            black_box(cache.set(key, test_preview.clone()).await)
        });
    });

    rt.block_on(async {
        cache
            .set("test_key".to_string(), test_preview.clone())
            .await;
    });

    group.bench_function("cache_read", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(cache.get("test_key").await) });
    });

    group.finish();
}

fn generate_test_preview() -> Preview {
    Preview {
        url: "https://example.com".to_string(),
        title: Some("Test Title".to_string()),
        description: Some("Test Description".to_string()),
        image_url: Some("https://example.com/image.jpg".to_string()),
        favicon: Some("https://example.com/favicon.ico".to_string()),
        site_name: Some("Example Site".to_string()),
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10));
    targets = bench_single_preview, bench_cache_operations
);
criterion_main!(benches);
