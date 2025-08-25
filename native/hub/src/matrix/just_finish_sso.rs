use matrix_sdk::{SessionMeta, SessionTokens};
use rinf::debug_print;

use crate::{
    matrix::client::{ArcMatrixClients, MatrixClient, OidcError},
    messages::*,
};

#[derive(Debug)]
pub enum Error {
    Oidc(OidcError),
}

pub async fn just_finish_sso(
    client: &mut MatrixClient,
    url: String,
) -> Result<(SessionMeta, SessionTokens), Error> {
    client
        .login_with_oidc_callback(url)
        .await
        .map_err(Error::Oidc)?;

    let tokens = client.0.oauth().user_session().map(|s| s.tokens);

    let meta = client.0.session_meta();

    match (meta, tokens) {
        (Some(meta), Some(tokens)) => Ok((meta.clone(), tokens)),
        _ => Err(Error::Oidc(OidcError::Cancelled)),
    }
}

pub async fn communicate(clients: ArcMatrixClients) {
    let receiver = JustFinishSso::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: JustFinishSso = dart_signal.message;
        debug_print!("JustFinishSso: received {message:?}");

        let mut c = clients.lock().await;
        let client = c.get_mut(&message.id);

        match client {
            Some(client) => match just_finish_sso(client, message.callback_url).await {
                Ok((meta, tokens)) => {
                    debug_print!("JustFinishSso: ok");
                    JustSsoTokens {
                        id: message.id,
                        device_id: meta.device_id.to_string(),
                        user_id: meta.user_id.to_string(),
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token.unwrap_or_default(),
                        error: Default::default(),
                    }
                    .send_signal_to_dart();
                }
                Err(err) => {
                    debug_print!("JustFinishSso: err {err:?}");
                    JustSsoTokens {
                        id: message.id,
                        device_id: Default::default(),
                        user_id: Default::default(),
                        access_token: Default::default(),
                        refresh_token: Default::default(),
                        error: match err {
                            Error::Oidc(err) => err.to_string(),
                        },
                    }
                    .send_signal_to_dart();
                }
            },
            None => {
                debug_print!(
                    "JustFinishSso: no client with associated {} was found",
                    &message.id
                );
                JustSsoTokens {
                    id: message.id,
                    device_id: Default::default(),
                    user_id: Default::default(),
                    access_token: Default::default(),
                    refresh_token: Default::default(),
                    error: "missing client".to_string(),
                }
                .send_signal_to_dart();
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::matrix::{client::OidcConfiguration, just_finish_sso::just_finish_sso};
//
//     #[tokio::test]
//     async fn test() {
//         let server = "efael.uz".to_string();
//         let url =
//             "uz.efael.app:/?state=kt-UW3AgpS0wQeG3Y98otA&code=5QQxgpV1SoJRnNfBiNDeaee2njCJNDz3"
//                 .to_string();
//         let tokens = just_finish_sso(url).await;
//
//         println!("{tokens:?}");
//
//         assert!(tokens.is_ok());
//     }
// }
