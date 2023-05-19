use poem::handler;
use poem::web::{Path, Query};

#[handler]
pub async fn download(Path(path): Path<String>) {
    todo!()
}

#[handler]
pub async fn upload(Path(path): Path<String>) {
    todo!()
}

#[handler]
pub async fn rename(Path(path): Path<String>, Query(name): Query<String>) {
    todo!()
}

#[handler]
pub async fn remove(Path(path): Path<String>) {
    todo!()
}
