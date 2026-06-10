use clnrm_core::validation::otel::ValidationSpanProcessor;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use opentelemetry::trace::{SpanContext, SpanId, Status, TraceFlags, TraceId, TraceState};
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks, SpanProcessor};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

fn create_mock_span_data(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1; 16]);
    let span_context = SpanContext::new(
        trace_id,
        SpanId::from_bytes([2; 8]),
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    );

    let start_time = SystemTime::now();
    let end_time = start_time + std::time::Duration::from_millis(10);

    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        parent_span_is_remote: false,
        span_kind: opentelemetry::trace::SpanKind::Internal,
        name: name.to_string().into(),
        start_time,
        end_time,
        attributes: vec![],
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        instrumentation_scope: InstrumentationScope::default(),
    }
}

fn bench_validation_span_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_span_processor");
    // Set a reasonable sample size for this heavy benchmark
    group.sample_size(10);

    group.bench_function("process_100k_spans_concurrently", |b| {
        b.iter_batched(
            || {
                let processor = Arc::new(ValidationSpanProcessor::new());
                let mut spans_batches = vec![];
                // 10 threads, 10k spans each = 100k spans concurrently
                for _ in 0..10 {
                    let mut batch = vec![];
                    for i in 0..10_000 {
                        batch.push(create_mock_span_data(&format!("span_{}", i)));
                    }
                    spans_batches.push(batch);
                }
                (processor, spans_batches)
            },
            |(processor, spans_batches)| {
                let mut handles = vec![];
                for batch in spans_batches {
                    let p = Arc::clone(&processor);
                    handles.push(thread::spawn(move || {
                        for span in batch {
                            p.on_end(span);
                        }
                    }));
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                // Verify all were processed
                let spans = processor.get_spans().unwrap();
                assert_eq!(spans.len(), 100_000);
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_validation_span_processor);
criterion_main!(benches);
