use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::{prelude::*, DefaultPlugins};
use bevy_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        Bytes, DefaultChannel, RenetClient, RenetServer,
    },
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    Ping(u32),
    Pong(u32),
}

impl From<Message> for Bytes {
    fn from(value: Message) -> Self {
        let mut v = Vec::new();
        ciborium::ser::into_writer(&value, &mut v).expect("serialization failed");
        println!("message serialization: {v:?}");
        v.into()
    }
}

impl From<Bytes> for Message {
    fn from(value: Bytes) -> Self {
        ciborium::de::from_reader(&*value).expect("deserialization failed")
    }
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

#[derive(Resource)]
struct ClientPingTimer(Timer);

impl Default for ClientPingTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(5), TimerMode::Repeating))
    }
}

fn client_send_pings(
    mut client: ResMut<RenetClient>,
    mut timer: ResMut<ClientPingTimer>,
    time: Res<Time>,
    mut counter: Local<u32>,
) {
    let just_finished = timer.0.tick(time.delta()).just_finished();
    if just_finished {
        client.send_message(DefaultChannel::ReliableOrdered, Message::Ping(*counter));
        *counter = counter.wrapping_add(1);
        println!("sending ping");
    }
}

fn client_recv_pings(mut client: ResMut<RenetClient>) {
    while let Some(msg) = client
        .receive_message(DefaultChannel::ReliableOrdered)
        .map(Message::from)
    {
        println!("Received (what should be) a pong: {msg:?}")
    }
}

fn server_respond_pongs(mut server: ResMut<RenetServer>) {
    for client in server.clients_id() {
        while let Some(msg) = server
            .receive_message(client, DefaultChannel::ReliableOrdered)
            .map(Message::from)
        {
            if let Message::Ping(id) = msg {
                server.send_message(client, DefaultChannel::ReliableOrdered, Message::Pong(id))
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let client_or_server = std::env::args()
        .nth(1)
        .expect("specify either server or client");
    if client_or_server == "client" {
        app.insert_resource(RenetClient::new(default()))
            .init_resource::<ClientPingTimer>()
            .insert_resource(client_transport())
            .add_system(client_recv_pings)
            .add_system(client_send_pings)
            .add_plugin(NetcodeClientPlugin)
            .add_plugin(RenetClientPlugin);
    } else if client_or_server == "server" {
        app.insert_resource(RenetServer::new(default()))
            .insert_resource(server_transport())
            .add_system(server_respond_pongs)
            .add_plugin(NetcodeServerPlugin)
            .add_plugin(RenetServerPlugin);
    } else {
        panic!("specify either server or client")
    }

    app.run();
}
