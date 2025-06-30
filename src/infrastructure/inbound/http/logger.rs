use std::env;

use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
};
use tracing::{Span, level_filters::LevelFilter};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_subscriber::EnvFilter;

pub struct CustomLogger;

impl RootSpanBuilder for CustomLogger {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let method = request.method().to_string();
        let route = request.uri().path();
        tracing::info_span!("http_request", method = %method, route = %route)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

impl CustomLogger {
    pub fn init_logger() {
        let level = env::var("RUST_LOG")
            .unwrap_or("INFO".to_string())
            .parse::<LevelFilter>()
            .unwrap();
        println!(" . . . Setting {level:?}");
        let filter = EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy();

        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}
