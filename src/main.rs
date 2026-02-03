use actix_web::{App, HttpResponse, HttpServer, Responder, web};


async fn post_program() -> impl Responder{
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main()  -> std::io::Result<()>{
    let addr = ("127.0.0.1", 8080);
    println!("app is bound to http://{}:{}",addr.0, addr.1);
    HttpServer::new(|| {
        App::new().route("/", web::post().to(post_program))
    }).bind(addr)?
    .run()
    .await

}
