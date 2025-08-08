use lambda_http::{Body, Error, IntoResponse, Request, RequestExt, Response};

pub async fn controller(event: Request) -> Result<Response<Body>, Error> {
    let first_name_opt = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("first_name"));

    match first_name_opt {
        Some(first_name) => Ok(format!("Verified ${first_name}").into_response().await),
        None => Ok(Response::builder()
            .status(400)
            .body("No name, can't verify".into())
            .expect("failed to render response")),
    }

}
