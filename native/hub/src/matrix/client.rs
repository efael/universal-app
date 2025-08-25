use matrix_sdk::{
    authentication::oauth::{
        error::OAuthAuthorizationCodeError,
        registration::{ApplicationType, ClientMetadata, Localized, OAuthGrantType},
        ClientId, ClientRegistrationData, OAuthAuthorizationData, OAuthError as SdkOAuthError,
    },
    ruma::OwnedDeviceId,
    Error,
};
use std::{collections::HashMap, sync::Arc};

use matrix_sdk::{
    ruma::{
        api::client::{
            discovery::get_authorization_server_metadata::v1::Prompt as RumaOidcPrompt,
            session::get_login_types,
        },
        serde::Raw,
    },
    sliding_sync::Version as SdkSlidingSyncVersion,
    Client,
};
use tokio::sync::Mutex;
use url::Url;

pub type ArcMatrixClients = Arc<Mutex<HashMap<String, MatrixClient>>>;

pub struct MatrixClient(pub Client);

#[derive(Debug)]
pub enum SlidingSyncVersion {
    None,
    Native,
}

impl From<SdkSlidingSyncVersion> for SlidingSyncVersion {
    fn from(value: SdkSlidingSyncVersion) -> Self {
        match value {
            SdkSlidingSyncVersion::None => Self::None,
            SdkSlidingSyncVersion::Native => Self::Native,
        }
    }
}

impl TryFrom<SlidingSyncVersion> for SdkSlidingSyncVersion {
    type Error = ();

    fn try_from(value: SlidingSyncVersion) -> Result<Self, Self::Error> {
        Ok(match value {
            SlidingSyncVersion::None => Self::None,
            SlidingSyncVersion::Native => Self::Native,
        })
    }
}

#[derive(Debug)]
pub struct HomeserverLoginDetails {
    pub url: String,
    pub sliding_sync_version: SlidingSyncVersion,
    pub supports_oidc_login: bool,
    pub supported_oidc_prompts: Vec<OidcPrompt>,
    pub supports_sso_login: bool,
    pub supports_password_login: bool,
}

#[derive(Debug)]
pub enum OidcPrompt {
    /// The Authorization Server should prompt the End-User to create a user
    /// account.
    ///
    /// Defined in [Initiating User Registration via OpenID Connect](https://openid.net/specs/openid-connect-prompt-create-1_0.html).
    Create,

    /// The Authorization Server should prompt the End-User for
    /// reauthentication.
    Login,

    /// The Authorization Server should prompt the End-User for consent before
    /// returning information to the Client.
    Consent,

    /// An unknown value.
    Unknown { value: String },
}

impl From<RumaOidcPrompt> for OidcPrompt {
    fn from(value: RumaOidcPrompt) -> Self {
        match value {
            RumaOidcPrompt::Create => Self::Create,
            value => match value.as_str() {
                "consent" => Self::Consent,
                "login" => Self::Login,
                _ => Self::Unknown {
                    value: value.to_string(),
                },
            },
        }
    }
}

impl From<OidcPrompt> for RumaOidcPrompt {
    fn from(value: OidcPrompt) -> Self {
        match value {
            OidcPrompt::Create => Self::Create,
            OidcPrompt::Consent => Self::from("consent"),
            OidcPrompt::Login => Self::from("login"),
            OidcPrompt::Unknown { value } => value.into(),
        }
    }
}
/// The configuration to use when authenticating with OIDC.
pub struct OidcConfiguration {
    /// The name of the client that will be shown during OIDC authentication.
    pub client_name: Option<String>,
    /// The redirect URI that will be used when OIDC authentication is
    /// successful.
    pub redirect_uri: String,
    /// A URI that contains information about the client.
    pub client_uri: String,
    /// A URI that contains the client's logo.
    pub logo_uri: Option<String>,
    /// A URI that contains the client's terms of service.
    pub tos_uri: Option<String>,
    /// A URI that contains the client's privacy policy.
    pub policy_uri: Option<String>,

    /// Pre-configured registrations for use with homeservers that don't support
    /// dynamic client registration.
    ///
    /// The keys of the map should be the URLs of the homeservers, but keys
    /// using `issuer` URLs are also supported.
    pub static_registrations: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error(
        "The homeserver doesn't provide an authentication issuer in its well-known configuration."
    )]
    NotSupported,
    #[error("Unable to use OIDC as the supplied client metadata is invalid.")]
    MetadataInvalid,
    #[error("The supplied callback URL used to complete OIDC is invalid.")]
    CallbackUrlInvalid,
    #[error("The OIDC login was cancelled by the user.")]
    Cancelled,

    #[error("An error occurred: {message}")]
    Generic { message: String },
}

impl From<SdkOAuthError> for OidcError {
    fn from(e: SdkOAuthError) -> OidcError {
        match e {
            SdkOAuthError::Discovery(error) if error.is_not_supported() => OidcError::NotSupported,
            SdkOAuthError::AuthorizationCode(OAuthAuthorizationCodeError::RedirectUri(_))
            | SdkOAuthError::AuthorizationCode(OAuthAuthorizationCodeError::InvalidState) => {
                OidcError::CallbackUrlInvalid
            }
            SdkOAuthError::AuthorizationCode(OAuthAuthorizationCodeError::Cancelled) => {
                OidcError::Cancelled
            }
            _ => OidcError::Generic {
                message: e.to_string(),
            },
        }
    }
}

impl From<Error> for OidcError {
    fn from(e: Error) -> OidcError {
        match e {
            Error::OAuth(e) => (*e).into(),
            _ => OidcError::Generic {
                message: e.to_string(),
            },
        }
    }
}

impl OidcConfiguration {
    pub(crate) fn redirect_uri(&self) -> Result<Url, OidcError> {
        Url::parse(&self.redirect_uri).map_err(|_| OidcError::CallbackUrlInvalid)
    }

    pub(crate) fn client_metadata(&self) -> Result<Raw<ClientMetadata>, OidcError> {
        let redirect_uri = self.redirect_uri()?;
        let client_name = self
            .client_name
            .as_ref()
            .map(|n| Localized::new(n.to_owned(), []));
        let client_uri = self.client_uri.localized_url()?;
        let logo_uri = self.logo_uri.localized_url()?;
        let policy_uri = self.policy_uri.localized_url()?;
        let tos_uri = self.tos_uri.localized_url()?;

        let metadata = ClientMetadata {
            // The server should display the following fields when getting the user's consent.
            client_name,
            logo_uri,
            policy_uri,
            tos_uri,
            ..ClientMetadata::new(
                ApplicationType::Native,
                vec![
                    OAuthGrantType::AuthorizationCode {
                        redirect_uris: vec![redirect_uri],
                    },
                    OAuthGrantType::DeviceCode,
                ],
                client_uri,
            )
        };

        Raw::new(&metadata).map_err(|_| OidcError::MetadataInvalid)
    }

    pub(crate) fn registration_data(&self) -> Result<ClientRegistrationData, OidcError> {
        let client_metadata = self.client_metadata()?;

        let mut registration_data = ClientRegistrationData::new(client_metadata);

        if !self.static_registrations.is_empty() {
            let static_registrations = self
                .static_registrations
                .iter()
                .filter_map(|(issuer, client_id)| {
                    let Ok(issuer) = Url::parse(issuer) else {
                        // tracing::error!("Failed to parse {issuer:?}");
                        return None;
                    };
                    Some((issuer, ClientId::new(client_id.clone())))
                })
                .collect();

            registration_data.static_registrations = Some(static_registrations);
        }

        Ok(registration_data)
    }
}

impl MatrixClient {
    pub async fn from_name_or_homeserver_url(
        name_or_homeserver_url: &str,
    ) -> Result<Self, matrix_sdk::ClientBuildError> {
        Client::builder()
            .server_name_or_homeserver_url(name_or_homeserver_url)
            .build()
            .await
            .map(Self)
    }

    pub async fn homeserver_login_details(&self) -> HomeserverLoginDetails {
        let oauth = self.0.oauth();
        let (supports_oidc_login, supported_oidc_prompts) = match oauth.server_metadata().await {
            Ok(metadata) => {
                let prompts = metadata
                    .prompt_values_supported
                    .into_iter()
                    .map(Into::into)
                    .collect();

                (true, prompts)
            }
            Err(_) => {
                // error!("Failed to fetch OIDC provider metadata: {error}");
                (false, Default::default())
            }
        };

        let login_types = self.0.matrix_auth().get_login_types().await.ok();
        let supports_password_login = login_types
            .as_ref()
            .map(|login_types| {
                login_types.flows.iter().any(|login_type| {
                    matches!(login_type, get_login_types::v3::LoginType::Password(_))
                })
            })
            .unwrap_or(false);
        let supports_sso_login = login_types
            .as_ref()
            .map(|login_types| {
                login_types
                    .flows
                    .iter()
                    .any(|login_type| matches!(login_type, get_login_types::v3::LoginType::Sso(_)))
            })
            .unwrap_or(false);
        let sliding_sync_version = self.0.sliding_sync_version().into();

        HomeserverLoginDetails {
            url: self.0.homeserver().into(),
            sliding_sync_version,
            supports_oidc_login,
            supported_oidc_prompts,
            supports_sso_login,
            supports_password_login,
        }
    }

    /// Requests the URL needed for opening a web view using OIDC. Once the web
    /// view has succeeded, call `login_with_oidc_callback` with the callback it
    /// returns. If a failure occurs and a callback isn't available, make sure
    /// to call `abort_oidc_auth` to inform the client of this.
    ///
    /// # Arguments
    ///
    /// * `oidc_configuration` - The configuration used to load the credentials
    ///   of the client if it is already registered with the authorization
    ///   server, or register the client and store its credentials if it isn't.
    ///
    /// * `prompt` - The desired user experience in the web UI. No value means
    ///   that the user wishes to login into an existing account, and a value of
    ///   `Create` means that the user wishes to register a new account.
    ///
    /// * `login_hint` - A generic login hint that an identity provider can use
    ///   to pre-fill the login form. The format of this hint is not restricted
    ///   by the spec as external providers all have their own way to handle the hint.
    ///   However, it should be noted that when providing a user ID as a hint
    ///   for MAS (with no upstream provider), then the format to use is defined
    ///   by [MSC4198]: https://github.com/matrix-org/matrix-spec-proposals/pull/4198
    ///
    /// * `device_id` - The unique ID that will be associated with the session.
    ///   If not set, a random one will be generated. It can be an existing
    ///   device ID from a previous login call. Note that this should be done
    ///   only if the client also holds the corresponding encryption keys.
    pub async fn url_for_oidc(
        &self,
        oidc_configuration: &OidcConfiguration,
        prompt: Option<OidcPrompt>,
        login_hint: Option<String>,
        device_id: Option<String>,
    ) -> Result<OAuthAuthorizationData, OidcError> {
        let registration_data = oidc_configuration.registration_data()?;
        let redirect_uri = oidc_configuration.redirect_uri()?;

        let device_id = device_id.map(OwnedDeviceId::from);

        let mut url_builder =
            self.0
                .oauth()
                .login(redirect_uri, device_id, Some(registration_data));

        if let Some(prompt) = prompt {
            url_builder = url_builder.prompt(vec![prompt.into()]);
        }
        if let Some(login_hint) = login_hint {
            url_builder = url_builder.login_hint(login_hint);
        }

        let data = url_builder.build().await?;

        Ok(data)
    }

    /// Completes the OIDC login process.
    pub async fn login_with_oidc_callback(&self, callback_url: String) -> Result<(), OidcError> {
        let url = Url::parse(&callback_url).or(Err(OidcError::CallbackUrlInvalid))?;

        self.0.oauth().finish_login(url.into()).await?;

        Ok(())
    }
}

trait OptionExt {
    /// Convenience method to convert an `Option<String>` to a URL and returns
    /// it as a Localized URL. No localization is actually performed.
    fn localized_url(&self) -> Result<Option<Localized<Url>>, OidcError>;
}

impl OptionExt for Option<String> {
    fn localized_url(&self) -> Result<Option<Localized<Url>>, OidcError> {
        self.as_deref().map(StrExt::localized_url).transpose()
    }
}

trait StrExt {
    /// Convenience method to convert a string to a URL and returns it as a
    /// Localized URL. No localization is actually performed.
    fn localized_url(&self) -> Result<Localized<Url>, OidcError>;
}

impl StrExt for str {
    fn localized_url(&self) -> Result<Localized<Url>, OidcError> {
        Ok(Localized::new(
            Url::parse(self).map_err(|_| OidcError::MetadataInvalid)?,
            [],
        ))
    }
}
