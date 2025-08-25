use crate::{
    matrix::client::{ArcMatrixClients, MatrixClient},
    messages::*,
};

pub async fn init_client(clients: ArcMatrixClients) {
    let receiver = CreateClient::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: CreateClient = dart_signal.message;

        rinf::debug_print!("{message:?}");
        match MatrixClient::from_name_or_homeserver_url(&message.name_or_homeserver_url).await {
            Ok(client) => {
                let mut clients = clients.lock().await;
                let id = uuid::Uuid::new_v4();
                let homeserver_login_details = client.homeserver_login_details().await;
                ClientCreated {
                    id: id.into(),
                    supports_oidc_login: homeserver_login_details.supports_oidc_login,
                    supports_password_login: homeserver_login_details.supports_password_login,
                    url: homeserver_login_details.url,
                }
                .send_signal_to_dart();
                clients.insert(id.into(), client);

                rinf::debug_print!("client created! {id:?}");
            }
            Err(error) => {
                rinf::debug_print!("{error:?}");
            }
        };
    }
}
