use std::env;
use std::fs;

use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
};
use tracing::{Span, level_filters::LevelFilter};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

use crate::infrastructure::inbound::http::ws_logger::WebSocketWriter;

pub struct CustomLogger {
    _file_guard: WorkerGuard,
    _ws_guard: WorkerGuard,
}

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
    pub fn init_logger() -> Self {
        let level = env::var("RUST_LOG")
            .unwrap_or("INFO".to_string())
            .parse::<LevelFilter>()
            .expect("RUST_LOG must be a valid level");

        println!("Setting logger with {level:?}");

        let filter = EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy();

        let log_dir = "logs";
        fs::create_dir_all(log_dir).ok();
        let file_appender = rolling::daily(log_dir, "app.log");
        let (file_non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let (ws_non_blocking, ws_guard) = tracing_appender::non_blocking(WebSocketWriter::new());

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(file_non_blocking).with_ansi(false)) // Capa para archivo
            .with(fmt::layer().with_writer(ws_non_blocking)) // Capa para WS
            .with(fmt::layer()) // Capa para stdout/stderr
            .init();

        Self {
            _file_guard: file_guard,
            _ws_guard: ws_guard,
        }
    }
}
