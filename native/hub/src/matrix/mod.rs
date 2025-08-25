mod client;
mod init_client;
mod just_finish_sso;
mod just_get_oidc_login_urls;

use crate::matrix::client::ArcMatrixClients;

pub async fn init() {
    let clients: ArcMatrixClients = Default::default();
    tokio::spawn(init_client::init_client(clients.clone()));
    tokio::spawn(just_get_oidc_login_urls::communicate(clients.clone()));
    tokio::spawn(just_finish_sso::communicate(clients));
}
