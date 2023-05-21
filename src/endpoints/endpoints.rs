use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next, Error};
//use crate::{db, logic::{self, generatePwdPHC}};
use crate::{db::db, models::usuario::{Usuario, TipoUsuario}};

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

pub async fn registrarUsuario(State(dbPool): State<sqlx::PgPool>, Json(mut payload): Json<Usuario>) -> Result<String, (StatusCode, String)> {
    payload.hashcontrasenna = payload.generatePwd().await;
    let res = db::insertarUsuario(payload, &dbPool).await;
    return match res {
        Ok(r) => match r.rows_affected() {
            0 => Err((StatusCode::BAD_REQUEST, "There was an error while creating the user".to_string())),
            1 => Ok("Done".to_string()),
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "This should not have happened.".to_string()))
        },
        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    };
}

//TODO: Return session id
pub async fn loginUsuario(State(dbPool): State<sqlx::PgPool>, Json(mut payload): Json<Usuario>) -> Result<String, (StatusCode, String)> {
    let usuario = db::buscarUsuario(&payload.nombreusuario, &dbPool).await;

    if usuario.is_err() == true {
        match usuario.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, "User does not exist".to_string())),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }

    //usuario.hashContrasenna currently contains the PHC
    let valid = payload.validatePwd(usuario.unwrap().hashcontrasenna).await;
    return match valid {
        false => Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string())),
        //True path should return session id
        true => Ok("Ok".to_string())
    };
}