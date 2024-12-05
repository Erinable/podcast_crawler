use criterion::{criterion_group, criterion_main, Criterion};
use podcast_crawler::infrastructure::initialize;
use tokio::runtime::Runtime;

fn app_init_benchmark(c: &mut Criterion) {
    // 创建 Tokio runtime
    let rt = Runtime::new().unwrap();

    c.bench_function("init app service", |b| {
        b.iter(|| {
            // 在每次迭代中运行异步代码
            rt.block_on(async {
                let state = initialize().await;
                state
            })
        });
    });
}

criterion_group!(benches, app_init_benchmark);
criterion_main!(benches);
