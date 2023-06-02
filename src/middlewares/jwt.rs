use std::sync::Arc;

use jwt_codec::prelude::VerifyingAlgorithm;
use jwt_codec::Codec;
use poem::async_trait;
use poem::http::StatusCode;
use poem::web::headers::HeaderMapExt;
use poem::web::headers::{authorization::Bearer, Authorization};
use poem::Endpoint;
use poem::Middleware;
use poem::Request;
use poem::{IntoResponse, Response};

use crate::models::permission::User;

#[derive(Debug)]
pub struct JwtVerifier<H> {
    codec: Arc<Codec<H>>,
}

#[derive(Debug)]
pub struct JwtVerifierEndpoint<H, E> {
    codec: Arc<Codec<H>>,
    ep: E,
}

impl<H, E> Middleware<E> for JwtVerifier<H>
where
    H: VerifyingAlgorithm + Send + Sync,
    E: Endpoint,
{
    type Output = JwtVerifierEndpoint<H, E>;

    fn transform(&self, ep: E) -> Self::Output {
        JwtVerifierEndpoint {
            codec: Arc::clone(&self.codec),
            ep,
        }
    }
}

#[async_trait]
impl<H, E> Endpoint for JwtVerifierEndpoint<H, E>
where
    H: VerifyingAlgorithm + Send + Sync,
    E: Endpoint,
{
    type Output = Response;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        let Some(claims) = req
            .headers()
            .typed_get::<Authorization<Bearer>>()
            .and_then(|Authorization(bear)| self
                .codec
                .parse_token::<User>(bear.token())
                .ok()
            )
        else {
            return Err(poem::Error::from_status(StatusCode::UNAUTHORIZED));
        };

        req.set_data(claims);
        self.ep.call(req).await.map(IntoResponse::into_response)
    }
}

impl<H> JwtVerifier<H> {
    #[inline]
    pub fn new(codec: Arc<Codec<H>>) -> Self {
        Self { codec }
    }
}
