use std::sync::Arc;

use jwt_codec::prelude::Hs256;
use jwt_codec::Claims;
use jwt_codec::Codec;
use poem::handler;
use poem::web::Data;
use poem::web::Json;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::QueryFilter;

use crate::db;
use crate::entity::prelude::Registry;
use crate::entity::registry;
use crate::entity::registry::InsertModel as UserModel;
use crate::models::permission::*;
use crate::reply::ReplyData;
use crate::reply::ReplyError;
use crate::utils::pswd;

#[handler]
pub async fn register(Json(mut user): Json<UserModel>) -> Result<ReplyData<()>, ReplyError> {
    user.password = pswd::hash(&user.password);
    user.into_active_model().insert(db::hdr()).await.unwrap();
    Ok(ReplyData(()))
}

#[handler]
pub async fn login(
    Json(login_form): Json<UserModel>,
    Data(codec): Data<&Arc<Codec<Hs256>>>,
) -> Result<ReplyData<Token>, ReplyError> {
    let Ok(Some(user)) = Registry::find()
        .filter(registry::Column::Username.eq(&login_form.username))
        .one(db::hdr())
        .await
    else {
        return Err(ReplyError::UserNotFound);
    };

    if pswd::verify(&login_form.password, &user.password) {
        let claims = Claims::new(User {
            name: login_form.username,
        })
        .valid_days(3);

        Ok(ReplyData(Token(codec.gen_token(&claims).unwrap())))
    } else {
        Err(ReplyError::IncorrectPassword)
    }
}

#[handler]
pub fn info(Data(user): Data<&User>) -> ReplyData<UserInfo> {
    ReplyData(UserInfo {
        username: user.name.clone(),
        roles: ["admin"],
    })
}
