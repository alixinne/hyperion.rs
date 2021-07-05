use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::{global::Global, models::ServerConfig};

pub mod boblight;
pub mod flat;
pub mod json;
pub mod proto;

pub struct ServerHandle {
    join_handle: JoinHandle<()>,
}

pub async fn bind<T, E, F, H>(
    name: &'static str,
    options: T,
    global: Global,
    handle_client: H,
) -> std::io::Result<ServerHandle>
where
    T: ServerConfig + Send + 'static,
    F: futures::Future<Output = Result<(), E>> + Send + 'static,
    E: From<std::io::Error> + std::fmt::Display + Send + 'static,
    H: Fn((TcpStream, SocketAddr), Global) -> F + Send + 'static,
{
    // Compute binding address
    let address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), options.port());

    // Setup listener
    let listener = TcpListener::bind(&address).await?;

    // Notify we are listening
    info!("{} server listening on {}", name, address);

    // Spawn accepting loop
    let join_handle = tokio::spawn(async move {
        let result: Result<(), _> = loop {
            match listener.accept().await {
                Ok(incoming) => {
                    tokio::spawn({
                        let peer_addr = incoming.1.clone();
                        let ft = handle_client(incoming, global.clone());

                        async move {
                            let result = ft.await;

                            match result {
                                Ok(_) => {
                                    info!("({}) client disconnected", peer_addr);
                                }
                                Err(error) => {
                                    error!("({}) client error:{}", peer_addr, error);
                                }
                            }
                        }
                    });
                }
                Err(error) => break Err(error),
            }
        };

        if let Err(error) = result {
            error!("{} server terminated: {}", name, error);
        }
    });

    Ok(ServerHandle { join_handle })
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}
