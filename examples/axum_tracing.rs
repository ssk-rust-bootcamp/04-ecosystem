use std::time::Duration;

use axum::routing::get;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator, Tracer},
    Resource,
};
use tokio::{
    join,
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_appender::non_blocking;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("/tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .with_writer(non_blocking)
        .pretty()
        .with_filter(LevelFilter::INFO);

    let tracer = init_tracker()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(opentelemetry)
        .init();
    let addr = "0.0.0.0:8080";
    let app = axum::Router::new().route("/", get(index_handler));
    let listener = TcpListener::bind(addr).await?;
    info!("Start server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
#[instrument]
async fn index_handler() -> &'static str {
    debug!("index handler started");
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    let s1 = sleep(Duration::from_millis(11));

    // spawn multiple tasks

    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    join!(s1, t1, t2, t3);
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, World!"
}

#[instrument]
async fn task1() {
    sleep(Duration::from_millis(10)).await;
}

#[instrument]
async fn task2() {
    sleep(Duration::from_millis(50)).await;
}

#[instrument]
async fn task3() {
    sleep(Duration::from_millis(30)).await;
}

fn init_tracker() -> anyhow::Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:58799"),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_event(64)
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "axum-tracing")])),
        )
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}
