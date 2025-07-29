use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_counter_vec, register_int_gauge,
    CounterVec, Encoder, HistogramVec, IntCounterVec, IntGauge, TextEncoder,
};
use std::time::Instant;

/// HTTP request duration histogram
pub static HTTP_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint", "status"]
    )
    .expect("Failed to register HTTP request duration metric")
});

/// HTTP request counter
pub static HTTP_REQUEST_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint", "status"]
    )
    .expect("Failed to register HTTP request counter")
});

/// Database query duration histogram
pub static DATABASE_QUERY_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "database_query_duration_seconds",
        "Database query duration in seconds",
        &["operation", "repository"]
    )
    .expect("Failed to register database query duration metric")
});

/// Database query counter
pub static DATABASE_QUERY_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "database_queries_total",
        "Total number of database queries",
        &["operation", "repository", "status"]
    )
    .expect("Failed to register database query counter")
});

/// Business metrics - items created
pub static ITEMS_CREATED_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!("items_created_total", "Total number of items created", &[])
        .expect("Failed to register items created counter")
});

/// Business metrics - items updated
pub static ITEMS_UPDATED_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!("items_updated_total", "Total number of items updated", &[])
        .expect("Failed to register items updated counter")
});

/// Business metrics - items deleted
pub static ITEMS_DELETED_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!("items_deleted_total", "Total number of items deleted", &[])
        .expect("Failed to register items deleted counter")
});

/// Active database connections
pub static DATABASE_CONNECTIONS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("database_connections_active", "Number of active database connections")
        .expect("Failed to register database connections gauge")
});

/// Initialize all metrics (called at startup to ensure registration)
pub fn init_metrics() {
    // Force lazy initialization and ensure metrics are registered
    Lazy::force(&HTTP_REQUEST_DURATION);
    Lazy::force(&HTTP_REQUEST_COUNTER);
    Lazy::force(&DATABASE_QUERY_DURATION);
    Lazy::force(&DATABASE_QUERY_COUNTER);
    Lazy::force(&ITEMS_CREATED_COUNTER);
    Lazy::force(&ITEMS_UPDATED_COUNTER);
    Lazy::force(&ITEMS_DELETED_COUNTER);
    Lazy::force(&DATABASE_CONNECTIONS);
}

/// Timer for measuring durations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Get metrics in Prometheus text format
pub fn get_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Track database query performance
pub fn track_database_query(operation: &str, repository: &str, success: bool, duration: f64) {
    let status = if success { "success" } else { "error" };

    DATABASE_QUERY_DURATION
        .with_label_values(&[operation, repository])
        .observe(duration);

    DATABASE_QUERY_COUNTER
        .with_label_values(&[operation, repository, status])
        .inc();
}

/// Track HTTP request
pub fn track_http_request(method: &str, endpoint: &str, status: u16, duration: f64) {
    let status_str = status.to_string();

    HTTP_REQUEST_DURATION
        .with_label_values(&[method, endpoint, &status_str])
        .observe(duration);

    HTTP_REQUEST_COUNTER
        .with_label_values(&[method, endpoint, &status_str])
        .inc();
}

/// Track business metrics
pub fn track_item_created() {
    ITEMS_CREATED_COUNTER.with_label_values(&[] as &[&str]).inc();
}

pub fn track_item_updated() {
    ITEMS_UPDATED_COUNTER.with_label_values(&[] as &[&str]).inc();
}

pub fn track_item_deleted() {
    ITEMS_DELETED_COUNTER.with_label_values(&[] as &[&str]).inc();
}
