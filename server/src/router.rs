use poem::get;
use poem::Route;

use crate::handlers::file_system;

#[rustfmt::skip]
pub fn new() -> Route {
    Route::new().
        at(
            "/fs/:path",
            get(file_system::download)
            .post(file_system::upload)
            .put(file_system::rename)
            .delete(file_system::remove)
        )
}
