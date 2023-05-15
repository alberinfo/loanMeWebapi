use axum::{routing::*, extract::Path, http::StatusCode, response::IntoResponse, Json, extract::State};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};

mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init(); //Initialize logging

    dotenvy::dotenv(); //load environment vars

    let dbPool = db::getDbConnection().await;

    if db::getTableCount() == 0 {
        sqlx::migrate!("./migrations").run(&dbPool).await?;
    }
    
    //All routes nested under /v0
    let v0: Router = axum::Router::new()
        .route("/Users", get(getUsuario))
        //.route("/Alumnos/:id", get(getAlumnoById))
        //.route("/Alumnos", post(insertAlumno))
        .with_state(dbPool);

    //Al routes nested under /api (i.e, /v0/*)
    let api: Router = axum::Router::new()
        .nest("/v0", v0);

    //All routes nested under / (i.e, /api/*)
    let app: Router = axum::Router::new()
        .nest("/api", api)
        .fallback(pageNotFound);

    let addr: SocketAddr = SocketAddr::from(([127,0,0,1], 4433));
    let config: RustlsConfig = RustlsConfig::from_pem_file(
        std::env::var("TLS_CERT_PATH").unwrap(),
        std::env::var("TLS_KEY_PATH").unwrap()
    )
    .await
    .unwrap();
    
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, "Page not found!");
}

async fn getUsuario(State(dbPool): State<sqlx::PgPool>, Json(payload): Json<db::Usuario>) -> Result<Json<db::Usuario>, (StatusCode, String)> {
    let result = db::buscarUsuario(payload.nombreusuario, &dbPool).await;
    return match result {
        Ok(r) => return Ok(Json(r)),
        Err(r) => return Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    }
}

//async fn insertarUsuario()