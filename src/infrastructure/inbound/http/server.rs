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

#[derive(Debug, Clone)]
pub struct HttpServerConfig<'a> {
    pub host: &'a str,
    pub port: u16,
}

pub struct HttpServer<'a> {
    server: actix_web::dev::Server,
    config: HttpServerConfig<'a>,
}

impl<'a> HttpServer<'a> {
    pub fn new<T: ClientBalanceService>(
        client_service: T,
        config: HttpServerConfig<'a>,
    ) -> Result<Self, anyhow::Error> {
        let arc_client_service = Arc::new(client_service);

        let server: actix_web::dev::Server = HttpServerAxum::new(move || {
            let client_service: web::Data<T> = web::Data::from(arc_client_service.clone());
            app_builder(client_service)
        })
        .bind((config.host, config.port))?
        .run();

        Ok(Self { server, config })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Starting HTTP server on {}:{} 🚀",
            self.config.host,
            self.config.port
        );
        self.server.await?;
        Ok(())
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
