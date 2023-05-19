use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next};
use crate::{db, logic::{self, generatePwdPHC}};

pub async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, "Page not found!");
}

//Bad name. Checks whether endpoint needs for the user to be logged in, and if so then checks whether or not the user _is_ logged in.
pub async fn validateCredentialsLayer(State(dbPool): State<sqlx::PgPool>, req: Request<axum::body::Body>, next: Next<axum::body::Body>) -> Response {
    //Camino actual
    let path = &req.uri().path().to_string();
    
    //Endpoints que no requieren validar al usuario.
    let skip_paths = vec!["/registrarUsuario", "/loginUsuario"]; //Añadir caminos a medida que sea necesario.
    for skip_path in skip_paths {
        if path.ends_with(skip_path) {
            return next.run(req).await;
        }
    }

    //do sth
    return next.run(req).await;
}

/*pub async fn getUsuario(State(dbPool): State<sqlx::PgPool>, Json(payload): Json<db::Usuario>) -> Result<Json<db::Usuario>, (StatusCode, String)> {
    let result = db::buscarUsuario(payload.nombreusuario, &dbPool).await;
    return match result {
        Ok(r) => Ok(Json(r)),
        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
    };
}*/

pub async fn registrarUsuario(State(dbPool): State<sqlx::PgPool>, Json(mut payload): Json<db::Usuario>) -> String {
    payload.hashcontrasenna = logic::generatePwdPHC(payload.hashcontrasenna.clone()).await;
    let res = db::insertarUsuario(payload, &dbPool).await;
    return match res {
        Ok(r) => match r.rows_affected() {
            0 => "There was an error while creating the user".to_string(),
            1 => "Done".to_string(),
            _ => "WTF".to_string()
        },
        Err(r) => r.to_string()
    };
}
