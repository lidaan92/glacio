use iron::{IronResult, Response, status};
use iron::headers::ContentType;
use serde::Serialize;
use serde_json;

/// Turns any serializable object into a JSON Iron response.
pub fn response<S: Serialize>(data: S) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, itry!(serde_json::to_string(&data))));
    response.headers.set(ContentType::json());
    Ok(response)
}
