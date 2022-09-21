use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use clap::Parser;
use pmtorrent::{FileDescription, FileRepo, Piece, RepoError};
use std::{net::SocketAddr, sync::Arc};
use tokio::fs::File;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    path: String,

    #[clap(short, long, default_value_t = 8080u16)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(args.path).await?;
    let file = pmtorrent::File::from_reader(file).await.unwrap();

    let mut repo = FileRepo::default();
    repo.add(file).expect("new file");

    let shared_state = Arc::new(repo);

    let app = Router::new()
        .route("/hashes", get(get_hashes))
        .route("/piece/:hashId/:pieceIdx", get(get_piece))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn get_hashes(
    Extension(repo): Extension<Arc<FileRepo>>,
) -> Result<Json<Vec<FileDescription>>, ApiError> {
    let res = repo.get_available();
    Ok(Json(res))
}

async fn get_piece(
    Extension(repo): Extension<Arc<FileRepo>>,
    Path((hash, piece)): Path<(String, usize)>,
) -> Result<Json<Piece>, ApiError> {
    let res = repo.get_piece(hash, piece)?;
    Ok(Json(res))
}

enum ApiError {
    Repo(RepoError),
}

impl From<RepoError> for ApiError {
    fn from(e: RepoError) -> Self {
        ApiError::Repo(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Repo(RepoError::DoesntExist) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Requested data does not exist",
            )
                .into_response(),
            ApiError::Repo(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}
