use goauth_rpc_client::v1::{
    auth_client::AuthClient, jwt_client::JwtClient, Empty, Response, UserRequest,
};
use heyo_rpc_client::rpc::Message;
use std::error::Error as stdError;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

use crate::{error::Error, handlers::user::UserPayload};
const ERR_TOKEN_EXPIRED: &str = "TokenExpiredError";

pub async fn jwt_status(identity_server_addr: &str, jwt: &str) -> Result<bool, Box<dyn stdError>> {
    let addr = Box::leak(identity_server_addr.to_string().clone().into_boxed_str());
    let channel = Channel::from_static(addr).connect().await?;
    let token: MetadataValue<_> = ("Authorization=".to_string() + jwt).parse()?;
    let mut client = JwtClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("set-cookie", token.clone());
        Ok(req)
    });

    let request = tonic::Request::new(Empty {});
    let response = client.status(request).await?;
    let inner_res = response.into_inner();
    if inner_res.code != 200 {
        if inner_res.message == ERR_TOKEN_EXPIRED {
            return Err(Box::new(Error::str(ERR_TOKEN_EXPIRED)));
        }
        return Err(Box::new(Error::str("Unknown error")));
    }
    Ok(true)
}

pub async fn user_signup(
    identity_server_addr: &str,
    user_payload: &UserPayload,
) -> Result<Response, Box<dyn stdError>> {
    println!("{:?}", &user_payload);
    let addr = Box::leak(identity_server_addr.to_string().clone().into_boxed_str());
    let request = tonic::Request::new(UserRequest {
        login: user_payload.login.clone(),
        password: user_payload.password.clone(),
        realm: user_payload.realm.clone(),
    });
    let channel = Channel::from_static(addr).connect().await?;
    Ok(AuthClient::new(channel).signup(request).await?.into_inner())
}

pub async fn user_login(
    identity_server_addr: &str,
    user_payload: &UserPayload,
) -> Result<(Response, String), Box<dyn stdError>> {
    let addr = Box::leak(identity_server_addr.to_string().clone().into_boxed_str());
    let request = tonic::Request::new(UserRequest {
        login: user_payload.login.clone(),
        password: user_payload.password.clone(),
        realm: user_payload.realm.clone(),
    });
    let channel = Channel::from_static(addr).connect().await?;
    let resp = AuthClient::new(channel).login(request).await?;
    let auth_header = match resp.metadata().get("set-cookie") {
        Some(md) => md.clone().to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };
    Ok((resp.into_inner(), auth_header))
}
