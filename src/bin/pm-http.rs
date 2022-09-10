use axum::{extract::Path, routing::get, Extension, Json, Router};
use pmtorrent::{FileDescription, FileRepo, Piece};
use std::{fs::File, io::Read, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() {
    // TODO: arguments
    // cleanup
    // Axum errors
    // docs
    // async?
    // devbox
    let mut repo = FileRepo::default();

    let mut file = File::open("../icons_rgb_circle.png").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let file = pmtorrent::File::new(&buf).unwrap();
    repo.add(file).expect("new file");

    let shared_state = Arc::new(repo);
    let app = Router::new()
        .route("/hashes", get(get_hashes))
        .route("/piece/:hashId/:pieceIdx", get(get_piece))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_hashes(Extension(repo): Extension<Arc<FileRepo>>) -> Json<Vec<FileDescription>> {
    let res = repo.get_available();
    Json(res)
}

async fn get_piece(
    Extension(repo): Extension<Arc<FileRepo>>,
    Path((hash, piece)): Path<(String, usize)>,
) -> Json<Piece> {
    let res = repo.get_piece(hash, piece).unwrap();
    Json(res)
}
