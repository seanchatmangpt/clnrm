use opentelemetry::global;
use opentelemetry::trace::{Tracer, TracerProvider};

pub fn test() {
    let _provider = global::tracer_provider();
}
