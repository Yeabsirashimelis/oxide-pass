use actix_web::{App, HttpResponse, HttpServer, Responder, web};

async fn run_program() -> impl Responder {
    println!("this is where the app runs");

    return HttpResponse::Ok().finish();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = ("127.0.0.1", 8001);
    println!("app is bound to http://{}: {}", addr.0, addr.1);
    HttpServer::new(move || App::new().route("/app", web::post().to(run_program)))
        .bind(addr)?
        .run()
        .await
}
