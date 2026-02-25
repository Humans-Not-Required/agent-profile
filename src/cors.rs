use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Method, Status},
    Request, Response,
};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response | Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        // Handle preflight OPTIONS requests
        if request.method() == Method::Options {
            request.set_method(Method::Get);
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request.headers().get_one("Origin").unwrap_or("*");

        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            origin.to_string(),
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PATCH, DELETE, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, X-API-Key, X-Manage-Token, X-Admin-Key, Authorization",
        ));
        response.set_header(Header::new(
            "Access-Control-Max-Age",
            "86400",
        ));
        response.set_header(Header::new(
            "Access-Control-Expose-Headers",
            "Content-Type",
        ));

        // Handle OPTIONS preflight
        if request.method() == Method::Get
            && request.uri().path().as_str() == "/"
            && request.headers().get_one("Access-Control-Request-Method").is_some()
        {
            response.set_status(Status::NoContent);
        }
    }
}
