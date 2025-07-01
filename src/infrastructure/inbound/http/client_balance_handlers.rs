use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};

use crate::{
    domain::port::inbound::client_balance_service::ClientBalanceService,
    infrastructure::inbound::http::{
        dto::{
            create_client::{CreateClientHttpRequestBody, CreateClientHttpResponseBody},
            get_client_balance::{
                GetClientBalanceHttpRequestPath, GetClientBalanceHttpResponseBody,
            },
            new_credit_transaction::{
                NewCreditTransactionHttpRequestBody, NewCreditTransactionHttpResponseBody,
            },
            new_debit_transaction::{
                NewDebitTransactionHttpRequestBody, NewDebitTransactionHttpResponseBody,
            },
            store_balances::StoreBalancesHttpResponseBody,
        },
        error::ApiError,
    },
};

pub async fn create_client<T: ClientBalanceService>(
    app_state: Data<T>,
    body: Json<CreateClientHttpRequestBody>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creating client");
    let req = body.into_inner();
    let req = req.try_into_domain()?;
    let client = app_state.get_ref().create_client(&req).await?;
    let response = CreateClientHttpResponseBody::from(client);
    Ok(HttpResponse::Created().json(response))
}

pub async fn get_client_balance<T: ClientBalanceService>(
    app_state: Data<T>,
    path: Path<GetClientBalanceHttpRequestPath>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Getting client info with balance info");
    let path = path.into_inner();
    let req = path.try_into_domain()?;
    let client = app_state.get_client_by_id(&req).await?;
    let client_balance = app_state.get_balance_by_client_id(&req).await?;
    let response = GetClientBalanceHttpResponseBody::from((client, client_balance));
    Ok(HttpResponse::Ok().json(response))
}

pub async fn new_credit_transaction<T: ClientBalanceService>(
    app_state: Data<T>,
    body: Json<NewCreditTransactionHttpRequestBody>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creating credit transaction");
    let req = body.into_inner();
    let req = req.try_into_domain()?;
    let client = app_state.get_ref().credit_balance(&req).await?;
    let response = NewCreditTransactionHttpResponseBody::from(client);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn new_debit_transaction<T: ClientBalanceService>(
    app_state: Data<T>,
    body: Json<NewDebitTransactionHttpRequestBody>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creating debit transaction");
    let req = body.into_inner();
    let req = req.try_into_domain()?;
    let client = app_state.get_ref().debit_balance(&req).await?;
    let response = NewDebitTransactionHttpResponseBody::from(client);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn store_balances<T: ClientBalanceService>(
    app_state: Data<T>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Storing balances");
    app_state.get_ref().store_balances().await?;
    let response = StoreBalancesHttpResponseBody::success_store();
    Ok(HttpResponse::Ok().json(response))
}

// with #[post("/create_client")] can't use generic type web::Data<T>:
// "cannot infer type of the type parameter `T` declared on the function `create_client`"
// https://github.com/actix/actix-web/issues/2866
#[macro_export]
macro_rules! CREATE_CLIENT_METHOD {
    ($service:ident) => {
        web::post()
            .to($crate::infrastructure::inbound::http::client_balance_handlers::create_client::<$service>)
    };
}
pub const CREATE_CLIENT_ROUTE: &str = "/create_client";

#[macro_export]
macro_rules! GET_CLIENT_BALANCE_METHOD {
    ($service:ident) => {
        web::get().to(
            $crate::infrastructure::inbound::http::client_balance_handlers::get_client_balance::<
                $service,
            >,
        )
    };
}
pub const GET_CLIENT_BALANCE_ROUTE: &str = "/client_balance/{user_id}";

#[macro_export]
macro_rules! NEW_CREDIT_TRANSACTION_METHOD {
    ($service:ident) => {
        web::post().to(
            $crate::infrastructure::inbound::http::client_balance_handlers::new_credit_transaction::<
                $service,
            >,
        )
    };
}
pub const NEW_CREDIT_TRANSACTION_ROUTE: &str = "/new_credit_transaction";

#[macro_export]
macro_rules! NEW_DEBIT_TRANSACTION_METHOD {
    ($service:ident) => {
        web::post().to(
            $crate::infrastructure::inbound::http::client_balance_handlers::new_debit_transaction::<
                $service,
            >,
        )
    };
}
pub const NEW_DEBIT_TRANSACTION_ROUTE: &str = "/new_debit_transaction";

#[macro_export]
macro_rules! STORE_BALANCES_METHOD {
    ($service:ident) => {
        web::post().to(
            $crate::infrastructure::inbound::http::client_balance_handlers::store_balances::<
                $service,
            >,
        )
    };
}
pub const STORE_BALANCES_ROUTE: &str = "/store_balances";
