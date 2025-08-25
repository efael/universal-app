use matrix_sdk::ClientBuildError;
use rinf::debug_print;
use url::Url;

use crate::{
    matrix::client::{ArcMatrixClients, MatrixClient, OidcConfiguration, OidcError, OidcPrompt},
    messages::*,
};

#[derive(Debug)]
pub enum Error {
    Oidc(OidcError),
    Client(ClientBuildError),
}

pub async fn get_oidc_url(
    url: String,
    oidc_configuration: &OidcConfiguration,
) -> Result<(MatrixClient, Url), Error> {
    let client = MatrixClient::from_name_or_homeserver_url(&url)
        .await
        .map_err(Error::Client)?;

    client
        .url_for_oidc(oidc_configuration, Some(OidcPrompt::Consent), None, None)
        .await
        .map(|data| (client, data.url))
        .map_err(Error::Oidc)
}

pub async fn communicate(clients: ArcMatrixClients) {
    let receiver = JustGetOidcUrls::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: JustGetOidcUrls = dart_signal.message;
        debug_print!("JustGetOidcUrls: received {message:?}");

        {
            let mut c = clients.lock().await;
            c.remove(&message.id);
        }

        match get_oidc_url(
            message.name_or_homeserver_url,
            &OidcConfiguration {
                client_name: Some(message.client_name),
                redirect_uri: message.redirect_uri,
                client_uri: message.client_uri,
                logo_uri: Some(message.logo_uri),
                tos_uri: Some(message.tos_uri),
                policy_uri: Some(message.policy_uri),
                static_registrations: Default::default(),
            },
        )
        .await
        {
            Ok((client, url)) => {
                let mut c = clients.lock().await;
                c.insert(message.id.clone(), client);
                debug_print!("JustGetOidcUrls: ok {url:?}");
                JustOidcUrls {
                    id: message.id,
                    url: url.into(),
                    error: "".to_string(),
                }
                .send_signal_to_dart();
            }
            Err(err) => {
                debug_print!("JustGetOidcUrls: err {err:?}");
                JustOidcUrls {
                    id: message.id,
                    url: "".to_string(),
                    error: match err {
                        Error::Oidc(err) => err.to_string(),
                        Error::Client(err) => err.to_string(),
                    },
                }
                .send_signal_to_dart();
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::{client::OidcConfiguration, just_get_oidc_login_urls::get_oidc_url};

    #[tokio::test]
    async fn test() {
        let url = "efael.uz".to_string();
        let oidc_configuration = &OidcConfiguration {
            client_name: Some("Efael".to_string()),
            redirect_uri: "uz.efael.app:/".to_string(),
            client_uri: "https://efael.uz".to_string(),
            logo_uri: Some("https://efael.uz".to_string()),
            tos_uri: Some("https://efael.uz".to_string()),
            policy_uri: Some("https://efael.uz".to_string()),
            static_registrations: Default::default(),
        };
        // let homeserver_login_details = client.homeserver_login_details().await;
        let url = get_oidc_url(url, oidc_configuration).await;

        assert!(url.is_ok());
    }
}
