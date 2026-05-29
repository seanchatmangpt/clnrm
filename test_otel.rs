fn main() {
    let mut provider = opentelemetry_sdk::trace::TracerProvider::builder().build();
    let res = provider.force_flush();
}
