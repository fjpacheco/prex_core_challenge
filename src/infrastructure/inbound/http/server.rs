use actix_web::{
    App, HttpServer as HttpServerAxum,
    dev::ServiceFactory,
    web::{self},
};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

use crate::{
    CREATE_CLIENT_METHOD, GET_CLIENT_BALANCE_METHOD, NEW_CREDIT_TRANSACTION_METHOD,
    NEW_DEBIT_TRANSACTION_METHOD, STORE_BALANCES_METHOD,
    domain::port::inbound::client_balance_service::ClientBalanceService,
    infrastructure::inbound::http::{
        client_balance_handlers::{
            CREATE_CLIENT_ROUTE, GET_CLIENT_BALANCE_ROUTE, NEW_CREDIT_TRANSACTION_ROUTE,
            NEW_DEBIT_TRANSACTION_ROUTE, STORE_BALANCES_ROUTE,
        },
        logger::CustomLogger,
    },
};

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;

pub struct HttpServer {
    server: actix_web::dev::Server,
    host: String,
    port: u16,
}

impl HttpServer {
    pub fn new<T: ClientBalanceService>(client_service: T) -> Result<Self, anyhow::Error> {
        let arc_client_service = Arc::new(client_service);

        let (host, port) = (Self::get_host(), Self::get_port());
        let server: actix_web::dev::Server = HttpServerAxum::new(move || {
            let client_service: web::Data<T> = web::Data::from(arc_client_service.clone());
            app_builder(client_service)
        })
        .bind((host.as_str(), port))?
        .run();

        Ok(Self { server, host, port })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!("Starting HTTP server on {}:{} 🚀", self.host, self.port);
        self.server.await?;
        Ok(())
    }

    pub fn get_port() -> u16 {
        let port = std::env::var("PORT").unwrap_or(DEFAULT_PORT.to_string());
        port.parse::<u16>().expect("PORT must be a number")
    }

    pub fn get_host() -> String {
        std::env::var("HOST").unwrap_or(DEFAULT_HOST.to_string())
    }
}

fn app_builder<T: ClientBalanceService>(
    client_service: web::Data<T>,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<
            tracing_actix_web::StreamSpan<actix_web::body::BoxBody>,
        >,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(client_service)
        .wrap(TracingLogger::<CustomLogger>::new())
        .route(CREATE_CLIENT_ROUTE, CREATE_CLIENT_METHOD!(T))
        .route(GET_CLIENT_BALANCE_ROUTE, GET_CLIENT_BALANCE_METHOD!(T))
        .route(
            NEW_CREDIT_TRANSACTION_ROUTE,
            NEW_CREDIT_TRANSACTION_METHOD!(T),
        )
        .route(
            NEW_DEBIT_TRANSACTION_ROUTE,
            NEW_DEBIT_TRANSACTION_METHOD!(T),
        )
        .route(STORE_BALANCES_ROUTE, STORE_BALANCES_METHOD!(T))
}
