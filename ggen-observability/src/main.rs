use ggen_observability::{
    LogEntry, LogLevel, LogCollector, TraceCollector, Trace, Span, SpanStatus,
    ObservabilityPipeline,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Observability System\n");
    println!("=========================\n");

    let mut pipeline = ObservabilityPipeline::new();

    println!("1. Creating request metrics...");
    let _request_counter = pipeline.metrics.create_counter("http_requests_total");
    let _error_counter = pipeline.metrics.create_counter("http_errors_total");
    let _latency_counter = pipeline.metrics.create_counter("http_request_duration_ms");

    pipeline.metrics.increment_counter("http_requests_total")?;
    pipeline.metrics.increment_counter("http_requests_total")?;
    pipeline.metrics.increment_counter("http_requests_total")?;
    pipeline.metrics.increment_counter("http_errors_total")?;

    println!("   ✓ HTTP Requests: {}", pipeline.metrics.get_counter("http_requests_total")?);
    println!("   ✓ HTTP Errors: {}", pipeline.metrics.get_counter("http_errors_total")?);
    println!();

    println!("2. Recording application logs...");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    pipeline.logs.log(LogEntry {
        timestamp: now,
        level: LogLevel::Info,
        service: "api-service".to_string(),
        message: "Server started on port 8080".to_string(),
        trace_id: "trace-api-001".to_string(),
        span_id: "span-001".to_string(),
        metadata: HashMap::from([
            ("port".to_string(), "8080".to_string()),
        ]),
    });

    pipeline.logs.log(LogEntry {
        timestamp: now + 100,
        level: LogLevel::Info,
        service: "api-service".to_string(),
        message: "Database connection established".to_string(),
        trace_id: "trace-api-001".to_string(),
        span_id: "span-002".to_string(),
        metadata: HashMap::from([
            ("db".to_string(), "postgres".to_string()),
        ]),
    });

    pipeline.logs.log(LogEntry {
        timestamp: now + 200,
        level: LogLevel::Warn,
        service: "cache-service".to_string(),
        message: "Cache eviction triggered".to_string(),
        trace_id: "trace-cache-001".to_string(),
        span_id: "span-003".to_string(),
        metadata: HashMap::new(),
    });

    pipeline.logs.log(LogEntry {
        timestamp: now + 300,
        level: LogLevel::Error,
        service: "worker-service".to_string(),
        message: "Task processing failed".to_string(),
        trace_id: "trace-worker-001".to_string(),
        span_id: "span-004".to_string(),
        metadata: HashMap::from([
            ("error".to_string(), "timeout".to_string()),
        ]),
    });

    println!("   ✓ Recorded 4 log entries");
    println!("   Info logs: {}", pipeline.logs.filter_by_level(LogLevel::Info).len());
    println!("   Warn logs: {}", pipeline.logs.filter_by_level(LogLevel::Warn).len());
    println!("   Error logs: {}", pipeline.logs.filter_by_level(LogLevel::Error).len());
    println!();

    println!("3. Recording distributed traces...");

    let trace1 = Trace {
        trace_id: "trace-api-001".to_string(),
        spans: vec![
            Span {
                span_id: "span-001".to_string(),
                trace_id: "trace-api-001".to_string(),
                parent_span_id: None,
                operation: "http.request".to_string(),
                service: "api-service".to_string(),
                start_time: now,
                end_time: now + 150,
                duration_ms: 150,
                status: SpanStatus::Ok,
                attributes: HashMap::from([
                    ("method".to_string(), "GET".to_string()),
                    ("path".to_string(), "/api/users".to_string()),
                ]),
            },
            Span {
                span_id: "span-002".to_string(),
                trace_id: "trace-api-001".to_string(),
                parent_span_id: Some("span-001".to_string()),
                operation: "db.query".to_string(),
                service: "postgres".to_string(),
                start_time: now + 10,
                end_time: now + 140,
                duration_ms: 130,
                status: SpanStatus::Ok,
                attributes: HashMap::from([
                    ("query".to_string(), "SELECT * FROM users".to_string()),
                ]),
            },
        ],
        start_time: now,
        end_time: now + 150,
        duration_ms: 150,
    };

    pipeline.traces.record_trace(trace1);

    let trace2 = Trace {
        trace_id: "trace-worker-001".to_string(),
        spans: vec![
            Span {
                span_id: "span-003".to_string(),
                trace_id: "trace-worker-001".to_string(),
                parent_span_id: None,
                operation: "task.process".to_string(),
                service: "worker-service".to_string(),
                start_time: now + 500,
                end_time: now + 2500,
                duration_ms: 2000,
                status: SpanStatus::Error,
                attributes: HashMap::from([
                    ("task_id".to_string(), "task-123".to_string()),
                ]),
            },
        ],
        start_time: now + 500,
        end_time: now + 2500,
        duration_ms: 2000,
    };

    pipeline.traces.record_trace(trace2);

    println!("   ✓ Recorded 2 traces");
    println!("   Total spans: 3");
    println!();

    println!("4. Trace analysis...");
    println!("   Slow traces (>1000ms): {}", pipeline.traces.slow_traces(1000).len());
    println!("   Error traces: {}", pipeline.traces.error_traces().len());

    let api_traces = pipeline.traces.traces_by_service("api-service");
    println!("   API Service traces: {}", api_traces.len());
    println!();

    println!("5. Detailed trace inspection...");
    if let Some(trace) = pipeline.traces.get_trace("trace-api-001") {
        println!("   Trace: trace-api-001");
        println!("     Duration: {} ms", trace.duration_ms);
        println!("     Spans: {}", trace.spans.len());
        for span in &trace.spans {
            let indent = if span.parent_span_id.is_some() { "       " } else { "     " };
            println!("{}├─ {} ({} ms)", indent, span.operation, span.duration_ms);
        }
    }
    println!();

    println!("6. Log querying by trace...");
    let trace_logs = pipeline.logs.filter_by_trace("trace-api-001");
    println!("   Logs for trace-api-001: {}", trace_logs.len());
    for log in trace_logs {
        println!("     [{}] {}", log.level.as_str(), log.message);
    }
    println!();

    println!("7. Service-specific logs...");
    let api_logs = pipeline.logs.filter_by_service("api-service");
    let worker_logs = pipeline.logs.filter_by_service("worker-service");
    println!("   API Service logs: {}", api_logs.len());
    println!("   Worker Service logs: {}", worker_logs.len());
    println!();

    println!("8. System health check...");
    let health = pipeline.health_check();
    println!("   Total logs: {}", health.total_logs);
    println!("   Total traces: {}", health.total_traces);
    println!("   Error logs: {}", health.error_logs);
    println!("   Error traces: {}", health.error_traces);
    println!("   Slow traces (>1000ms): {}", health.slow_traces);
    println!();

    println!("9. Performance summary...");
    let all_traces = pipeline.traces.get_traces();
    if !all_traces.is_empty() {
        let total_duration: u64 = all_traces.iter().map(|t| t.duration_ms).sum();
        let avg_duration = total_duration / all_traces.len() as u64;
        println!("   Average trace duration: {} ms", avg_duration);

        let fastest = all_traces.iter().min_by_key(|t| t.duration_ms).unwrap();
        let slowest = all_traces.iter().max_by_key(|t| t.duration_ms).unwrap();
        println!("   Fastest trace: {} ms", fastest.duration_ms);
        println!("   Slowest trace: {} ms", slowest.duration_ms);
    }
    println!();

    println!("10. Log levels distribution...");
    let debug_count = pipeline.logs.filter_by_level(LogLevel::Debug).len();
    let info_count = pipeline.logs.filter_by_level(LogLevel::Info).len();
    let warn_count = pipeline.logs.filter_by_level(LogLevel::Warn).len();
    let error_count = pipeline.logs.filter_by_level(LogLevel::Error).len();

    println!("   Debug: {}", debug_count);
    println!("   Info: {}", info_count);
    println!("   Warn: {}", warn_count);
    println!("   Error: {}", error_count);
    println!();

    println!("=========================");
    println!("Observability demo complete! ✓");
    Ok(())
}
