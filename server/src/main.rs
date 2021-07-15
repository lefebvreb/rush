use warp::Filter;

// The port the server listens on.
const PORT: u16 = 5050;

#[tokio::main]
async fn main() {
    // Initialize the chess library.
    chess::init();

    // For index.html.
    let index = warp::get()
        .and(warp::fs::dir("www/public"));

    // For wasm files.
    let assets = warp::path("assets")
        .and(warp::fs::dir("www/public/build/assets"));

    // The routes object.
    let routes = index.or(assets);

    // Launch the server.
    println!("Launching server @ http://localhost:{}", PORT);
    warp::serve(routes)
        .run(([127, 0, 0, 1], PORT))
        .await;
}
