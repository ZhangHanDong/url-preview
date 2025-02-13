use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use url_preview::{
    FetchResult, Fetcher, FetcherConfig, Preview, PreviewError, PreviewService,
    PreviewServiceConfig,
};

// Mock data for consistent benchmarking
const MOCK_HTML: &str = r#"
<!DOCTYPE html>
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
</html>
"#;

// Test URLs for benchmarking
const TEST_URLS: &[&str] = &[
    "https://test1.example.com",
    "https://test2.example.com",
    "https://test3.example.com",
    "https://test4.example.com",
    "https://test5.example.com",
];

const BATCH_SIZES: &[usize] = &[2, 5, 10];
const CONCURRENT_LIMITS: &[usize] = &[3, 5, 10];

// Create a mock preview for testing
fn create_mock_preview(url: &str) -> Preview {
    Preview {
        url: url.to_string(),
        title: Some("Test Title".to_string()),
        description: Some("Test Description".to_string()),
        image_url: Some("https://example.com/image.jpg".to_string()),
        favicon: Some("https://example.com/favicon.ico".to_string()),
        site_name: Some("Test Site".to_string()),
    }
}

// Mock fetcher that returns consistent results without network calls
#[derive(Clone)]
struct MockFetcher;

impl MockFetcher {
    async fn fetch(&self, _url: &str) -> Result<FetchResult, PreviewError> {
        // Simulate network latency
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(FetchResult::Html(MOCK_HTML.to_string()))
    }

    async fn fetch_batch(&self, urls: Vec<&str>) -> Result<Vec<FetchResult>, PreviewError> {
        // Simulate batch processing with consistent latency
        tokio::time::sleep(Duration::from_millis(50 * urls.len() as u64)).await;
        Ok(urls
            .iter()
            .map(|_| FetchResult::Html(MOCK_HTML.to_string()))
            .collect())
    }
}

fn bench_batch_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_processing");

    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(20))
        .warm_up_time(Duration::from_secs(5));

    let mock_fetcher = MockFetcher;

    for &size in BATCH_SIZES {
        let urls: Vec<&str> = TEST_URLS.iter().take(size).copied().collect();

        group.bench_with_input(BenchmarkId::new("batch_size", size), &urls, |b, urls| {
            let fetcher = mock_fetcher.clone();
            b.to_async(&rt)
                .iter(|| async { black_box(fetcher.fetch_batch(urls.to_vec()).await.unwrap()) });
        });
    }

    group.finish();
}

fn bench_concurrent_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_processing");

    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(20))
        .warm_up_time(Duration::from_secs(5));

    for &limit in CONCURRENT_LIMITS {
        let mock_fetcher = MockFetcher;

        group.bench_with_input(
            BenchmarkId::new("concurrent_limit", limit),
            &TEST_URLS,
            |b, urls| {
                let fetcher = mock_fetcher.clone();
                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = urls
                        .iter()
                        .map(|&url| {
                            let fetcher = fetcher.clone();
                            async move { fetcher.fetch(url).await }
                        })
                        .collect();

                    black_box(join_all(futures).await)
                });
            },
        );
    }

    group.finish();
}

fn bench_processing_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("processing_strategies");

    let batch_size = 5;
    let urls: Vec<&str> = TEST_URLS.iter().take(batch_size).copied().collect();
    let mock_fetcher = MockFetcher;

    // Benchmark batch processing
    group.bench_function("batch_strategy", |b| {
        let fetcher = mock_fetcher.clone();
        b.to_async(&rt)
            .iter(|| async { black_box(fetcher.fetch_batch(urls.clone()).await.unwrap()) });
    });

    // Benchmark concurrent processing
    group.bench_function("concurrent_strategy", |b| {
        let fetcher = mock_fetcher.clone();
        b.to_async(&rt).iter(|| async {
            let futures: Vec<_> = urls
                .iter()
                .map(|&url| {
                    let fetcher = fetcher.clone();
                    async move { fetcher.fetch(url).await }
                })
                .collect();

            black_box(join_all(futures).await)
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    targets = bench_batch_processing,
             bench_concurrent_processing,
             bench_processing_strategies
}
criterion_main!(benches);
