use crate::{client::{self, TcpPackage, TcpUpdate}, server, Game, GameUpdateData, LobbyUpdateData};

#[test]
fn it_works() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async move {
        server::listen("127.0.0.1:9983", Some(())).await.expect("Server failed to listen");
    });
    rt.block_on(async move {
        let msg = "This 1$ @ t€$t m€$$@ge!".to_string();
        let (socket, _lobby) = client::ConnectionSocket::build("127.0.0.1:9983", "0.0.0.0:9983", "tester".into()).await.expect("Failed to get ConnectionSocket");
        socket.tcp_send.send(TcpPackage::Message(msg.clone())).expect("Failed to send LobbyConnect request");
        let update = socket.tcp_recv.recv().expect("Failed to receive TcpUpdate!");

        assert_eq!(update, TcpUpdate::LobbyUpdate(LobbyUpdateData::Message { sender: 0, length: msg.len() as u8, content: msg }));

        socket.tcp_send.send(TcpPackage::GameCreation { name: "testWorld".to_string(), with_password: false }).expect("Failed to send GameCreation TcpPackage");
        let update = socket.tcp_recv.recv().expect("Failed to receive TcpUpdate! (#2)");

        assert_eq!(update, TcpUpdate::GameUpdate(GameUpdateData::Creation(Game {
            game_id: 0,
            host_id: 0,
            password: false,
            game_name: "testWorld".to_string(),
            clients: vec![0],
        })));

        socket.tcp_send.send(TcpPackage::GameWorld(SCENE_STRING.to_string())).expect("Failed to send GameWorld as TcpPackage");
        let update = socket.tcp_recv.recv().expect("Failed to receive TcpUpdate! (#3)");

        assert_eq!(update, TcpUpdate::GameWorld(SCENE_STRING.to_string()));
    });
}

const SCENE_STRING: &str = r#"(
  resources: {},
  entities: {
    8589934593: (
      components: {
        "bevy_transform::components::transform::Transform": (
          translation: (
            x: -0.1215124,
            y: 3.9994566,
            z: -0.121507265,
          ),
          rotation: (
            x: 0.0,
            y: -0.7028805,
            z: 0.0,
            w: 0.71130794,
          ),
          scale: (
            x: 0.4,
            y: 0.4,
            z: 0.4,
          ),
        ),
        "yggdrasil::game::components::Player": (
          base_velocity: 10.0,
          name: "Jon",
        ),
        "yggdrasil::game::components::Health": (
          value: 5,
        ),
      },
    ),
    8589934598: (
      components: {
        "bevy_transform::components::transform::Transform": (
          translation: (
            x: 30.0,
            y: 3.9994566,
            z: 0.0,
          ),
          rotation: (
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
          ),
          scale: (
            x: 0.4,
            y: 0.4,
            z: 0.4,
          ),
        ),
        "yggdrasil::game::components::Health": (
          value: 4,
        ),
        "yggdrasil::game::components::Npc": (),
      },
    ),
  },
)"#;
