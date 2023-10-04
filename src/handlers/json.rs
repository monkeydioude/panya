use rocket::serde::json::Json;

pub fn json_error<T>(err: String) -> Json<Vec<T>> {
    error!("{}", err);
    Json(Vec::<T>::new())
}

pub fn json_warn<T>(msg: String) -> Json<Vec<T>> {
    warn!("{}", msg);
    Json(Vec::<T>::new())
}
