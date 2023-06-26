use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{prelude::*, DefaultPlugins};
use bevy_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        RenetClient, RenetServer,
    },
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};

#[derive(Debug)]
enum Message {
    Ping(u32),
    Pong(u32),
}

const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

fn server_transport() -> NetcodeServerTransport {
    let socket = UdpSocket::bind(SERVER_ADDR).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        max_clients: 64,
        protocol_id: 0,
        public_addr: SERVER_ADDR,
        authentication: ServerAuthentication::Unsecure,
    };

    NetcodeServerTransport::new(current_time, server_config, socket).unwrap()
}

fn client_transport() -> NetcodeClientTransport {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id: u64 = 0;
    let authentication = ClientAuthentication::Unsecure {
        server_addr: SERVER_ADDR,
        client_id,
        user_data: None,
        protocol_id: 0,
    };

    NetcodeClientTransport::new(current_time, authentication, socket).unwrap()
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let client_or_server = std::env::args()
        .nth(1)
        .expect("specify either server or client");
    if client_or_server == "client" {
        app.insert_resource(RenetClient::new(default()))
            .insert_resource(client_transport())
            .add_plugin(NetcodeClientPlugin)
            .add_plugin(RenetClientPlugin);
    } else if client_or_server == "server" {
        app.insert_resource(RenetServer::new(default()))
            .insert_resource(server_transport())
            .add_plugin(NetcodeServerPlugin)
            .add_plugin(RenetServerPlugin);
    } else {
        panic!("specify either server or client")
    }

    app.run();
}
