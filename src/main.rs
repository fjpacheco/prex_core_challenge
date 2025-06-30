use prex_core_challenge::infrastructure::inbound::http::logger::CustomLogger;
use prex_core_challenge::infrastructure::inbound::http::server::HttpServerConfig;
use prex_core_challenge::infrastructure::outbound::{
    file_exporter::FileExporter, in_memory::InMemoryRepository,
};
use prex_core_challenge::{
    application::client_balance_service::Service, infrastructure::inbound::http::server::HttpServer,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    CustomLogger::init_logger();

    let file_exporter = FileExporter::new().await?;

    let in_memory_repository = InMemoryRepository::new();

    let service_client = Service::new(in_memory_repository, file_exporter);

    let server = HttpServer::new(
        service_client,
        HttpServerConfig {
            host: "127.0.0.1",
            port: 8080,
        },
    )?;

    server.run().await?;
    tracing::info!("Goodbye 👋");
    Ok(())
}
