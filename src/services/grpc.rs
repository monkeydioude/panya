use goauth_rpc_client::v1::{jwt_client::JwtClient, Empty};
use std::error::Error as stdError;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

use crate::error::Error;
const ERR_TOKEN_EXPIRED: &str = "TokenExpiredError";

pub async fn jwt_status(
    identity_server_addr: &'static str,
    jwt: &str,
) -> Result<bool, Box<dyn stdError>> {
    let channel = Channel::from_static(identity_server_addr).connect().await?;
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
