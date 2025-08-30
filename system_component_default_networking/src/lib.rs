use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{Event, GameState},
    },
    dumpster_engine::{GameMode, NetworkModes},
    system::{
        system_component::SystemComponent,
        system_components::system_component_networking::SystemComponentNetworking,
        system_game_state::{from_bytes, to_bytes},
    },
};
use message_io::node::NodeEvent;
use message_io::{
    network::Endpoint,
    node::{self, NodeListener},
};
use message_io::{
    network::{NetEvent, Transport},
    node::NodeHandler,
};
use std::sync::{Arc, Mutex};

pub struct SystemComponentDefaultNetworking {
    network_mode: NetworkModes,
    node_handler: NodeHandler<Signal>,
    endpoints: Arc<Mutex<Vec<Endpoint>>>,
    incoming_events: Arc<Mutex<Vec<Event>>>,
}

impl SystemComponentNetworking for SystemComponentDefaultNetworking {}
impl SystemComponentDefaultNetworking {
    pub fn new() -> Box<SystemComponentDefaultNetworking> {
        let (handler, _) = node::split::<Signal>();

        Box::new(SystemComponentDefaultNetworking {
            network_mode: NetworkModes::Offline,
            node_handler: handler,
            endpoints: Arc::new(Mutex::new(Vec::new())),
            incoming_events: Arc::new(Mutex::new(Vec::new())),
        })
    }
    fn init_offline(&mut self) {
        println!("init offline");
    }
    fn init_online_host(&mut self, handler: &NodeHandler<Signal>, listener: NodeListener<Signal>) {
        println!("init host");
        // Listen for TCP, UDP and WebSocket messages at the same time.
        handler
            .network()
            .listen(Transport::FramedTcp, "0.0.0.0:3042")
            .unwrap();
        handler
            .network()
            .listen(Transport::Udp, "0.0.0.0:3043")
            .unwrap();
        handler
            .network()
            .listen(Transport::Ws, "0.0.0.0:3044")
            .unwrap();

        let endpoints = self.endpoints.clone();
        thread::spawn(move || {
            listener.for_each(move |event| match event {
                NodeEvent::Network(net_event) => match net_event {
                    NetEvent::Accepted(endpoint, _) => {
                        let Ok(mut guard) = endpoints.lock() else {
                            panic!("Failed to lock");
                        };
                        guard.push(endpoint);
                        println!("Client Connected");
                    }
                    NetEvent::Message(_, _) => {
                        println!("Received A Message");
                    }
                    NetEvent::Connected(endpoint, _) => {
                        let Ok(mut guard) = endpoints.lock() else {
                            panic!("Failed to lock");
                        };
                        guard.push(endpoint);
                        println!("Client Connected");
                    }
                    NetEvent::Disconnected(endpoint) => {
                        let Ok(mut guard) = endpoints.lock() else {
                            panic!("Failed to lock");
                        };
                        guard.retain(|&e| e != endpoint);
                        println!("Client disconnected");
                    }
                },
                NodeEvent::Signal(signal) => match signal {},
            });
        });
    }
    fn init_online_peer(&mut self, handler: &NodeHandler<Signal>, listener: NodeListener<Signal>) {
        println!("init peer");

        // You can change the transport to Udp or Ws (WebSocket).
        let (server, _) = handler
            .network()
            .connect(Transport::FramedTcp, "127.0.0.1:3042")
            .unwrap();

        let endpoints = self.endpoints.clone();
        let incoming_events = self.incoming_events.clone();
        thread::spawn(move || {
            listener.for_each(move |event| match event {
                NodeEvent::Network(net_event) => match net_event {
                    NetEvent::Accepted(_, _) => {
                        println!("Client connected")
                    }
                    NetEvent::Message(_, data) => {
                        let event = from_bytes::<Event>(data);
                        let Ok(mut guard) = incoming_events.lock() else {
                            panic!("Failed to lock");
                        };

                        guard.push(event);
                        // println!("Received A Message with id {} ", event.id);
                    }
                    NetEvent::Connected(endpoint, _) => {
                        let Ok(mut guard) = endpoints.lock() else {
                            panic!("Failed to lock");
                        };
                        guard.push(endpoint);
                        println!("Client Connected");
                    }
                    NetEvent::Disconnected(endpoint) => {
                        let Ok(mut guard) = endpoints.lock() else {
                            panic!("Failed to lock");
                        };
                        guard.retain(|&e| e != endpoint);
                        println!("Client disconnected");
                    }
                },
                NodeEvent::Signal(signal) => match signal {},
            });
        });
    }
    fn tick_offline(&mut self, game_state: &mut GameState, _: &mut EventQueue) {}
    fn tick_online_host(&mut self, game_state: &mut GameState, _: &mut EventQueue) {
        let Ok(guard) = self.endpoints.lock() else {
            println!("couldnot lock");
            return;
        };

        let endpoints = guard.as_slice();
        let events = game_state.get_network_sync_events();
        // println!("sending {} messages to {} peers", events.len(), endpoints.len());

        for event in &events {
            for endpoint in endpoints {
                println!("sent to endpoint!");

                self.node_handler
                    .network()
                    .send(endpoint.clone(), to_bytes::<Event>(&event).as_slice());
            }
        }
    }
    fn tick_online_peer(&mut self, game_state: &mut GameState, _: &mut EventQueue) {
        let Ok(mut guard) = self.incoming_events.lock() else {
            println!("couldnot lock");
            return;
        };

        game_state.apply_network_sync_events(guard.to_vec());

        guard.clear();
    }
}
impl SystemComponent for SystemComponentDefaultNetworking {
    fn order(&self) -> i32 {
        10000
    }
    fn init(&mut self, _: &mut GameState) {
        println!("init networking");
    }
    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue) {
        match self.network_mode {
            NetworkModes::Offline => self.tick_offline(game_state, event_queue),
            NetworkModes::OnlineHost => self.tick_online_host(game_state, event_queue),
            NetworkModes::OnlinePeer => self.tick_online_peer(game_state, event_queue),
        }
    }
    fn set_game_mode(&mut self, game_mode: &GameMode) {
        let (handler, listener) = node::split::<Signal>();

        println!("set game mode");
        match game_mode.network_mode {
            NetworkModes::Offline => self.init_offline(),
            NetworkModes::OnlineHost => self.init_online_host(&handler, listener),
            NetworkModes::OnlinePeer => self.init_online_peer(&handler, listener),
        }

        self.network_mode = game_mode.network_mode.clone();
        self.node_handler = handler;
    }
}

enum Signal {}
use std::thread;

// #[derive(Clone, seri)]
// pub struct Payload {
//     pub payload: PayloadTypes,
// }
// pub enum PayloadTypes {
//     Message(String),
//     GameState(Event),
//     GameEvent,
// }
