use bevy::{prelude::*, DefaultPlugins};
use bevy_renet::{
    renet::{RenetClient, RenetServer},
    RenetClientPlugin, RenetServerPlugin,
};

enum Message {
    Left,
    Right,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let client_or_server = std::env::args()
        .nth(1)
        .expect("specify either server or client");
    if client_or_server == "client" {
        app.insert_resource(RenetClient::new(default()));
        app.add_plugin(RenetClientPlugin);
    } else if client_or_server == "server" {
        app.insert_resource(RenetServer::new(default()));
        app.add_plugin(RenetServerPlugin);
    } else {
        panic!("specify either server or client")
    }

    app.run();
}
