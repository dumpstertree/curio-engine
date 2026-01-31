use curio_core::{
    built_in::record::sys_record_network::SysRecordNetwork,
    collections::{event_queue::EventQueue, game_mode::GameMode, game_state::GameState, network_modes::NetworkModes, state_ownerships::StateOwnerships, state_sync_event::StateSyncEvent},
    system::{system_component::SystemComponent, system_components::system_component_networking::SystemComponentNetworking},
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
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    vec,
};

pub struct SystemComponentDefaultNetworking {
    network_mode: NetworkModes,
    node_handler: NodeHandler<Signal>,
    endpoints: Arc<Mutex<Vec<Endpoint>>>,
    incoming_events: Arc<Mutex<Vec<StateSyncEvent>>>,
}

impl SystemComponentNetworking for SystemComponentDefaultNetworking {}
impl SystemComponentDefaultNetworking {
    pub fn new() -> Box<SystemComponentDefaultNetworking> {
        let (handler, _) = node::split::<Signal>();

        Box::new(SystemComponentDefaultNetworking {
            network_mode: NetworkModes::LocalHost,
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
        let (_server, _) = handler
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
                        let event = from_bytes::<StateSyncEvent>(data);
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
    fn tick_offline(&mut self, _game_state: &mut GameState, _: &mut EventQueue) {}
    fn tick_online_host(&mut self, game_state: &mut GameState, _: &mut EventQueue) {
        let Ok(guard) = self.endpoints.lock() else {
            println!("couldnot lock");
            return;
        };

        let endpoints = guard.as_slice();
        let events = game_state.try_drain_network_sync_events();
        // println!("sending {} messages to {} peers", events.len(), endpoints.len());

        for event in &events {
            for endpoint in endpoints {
                println!("sent to endpoint!");

                self.node_handler
                    .network()
                    .send(endpoint.clone(), to_bytes::<StateSyncEvent>(&event).as_slice());
            }
        }
    }
    fn tick_online_peer(&mut self, game_state: &mut GameState, _: &mut EventQueue) {
        let Ok(mut guard) = self.incoming_events.lock() else {
            println!("couldnot lock");
            return;
        };

        game_state.try_apply_network_sync_events(&guard.to_vec());

        guard.clear();
    }
}
impl SystemComponent for SystemComponentDefaultNetworking {
    fn order(&self) -> i32 {
        10000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {
        println!("init networking");
    }
    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // save all pending changes and apply them at the end
        let mut pending_changes: HashMap<usize, Vec<StateSyncEvent>> = HashMap::new();

        // iterate over each gamestate creating list of pending changes
        for i in 0..game_state.len() {
            // get the gamestate we are using
            let Some(game_state_a) = game_state.get_mut(i) else {
                println!("Failed to get GameState at index: {}", i);
                continue;
            };

            // get the network capabilities of the gamestate. This is not required and can be silently passed
            let Some(game_state_a_network_capabalities) = game_state_a.network_capabilities.clone() else {
                continue;
            };

            // drain all network sync events
            let drained_sync_events = game_state_a.try_drain_network_sync_events();
            // iterate over each sync event
            for sync_event in &drained_sync_events {
                // if this state change is instance only we can skip it
                if sync_event.ownership == StateOwnerships::Instance {
                    continue;
                }
                // iterate over all gamestates again
                for j in 0..game_state.len() {
                    // if i and j are equal that means we are syncing with ourself and can skip
                    if i == j {
                        continue;
                    }
                    // get the second GameState we are using
                    let Some(game_state_b) = game_state.get_mut(i) else {
                        println!("Failed to get GameState at index: {}", i);
                        continue;
                    };

                    // get the network capabilities of the GameState. This is not required and can be silently passed
                    let Some(game_state_b_network_capabalities) = game_state_b.network_capabilities.clone() else {
                        continue;
                    };

                    // compare privilege levels and make sure the ones trying to override are higher
                    if game_state_a_network_capabalities.privilege >= game_state_b_network_capabalities.privilege {
                        // get the list of pending changes and add this sync event. if the list is not yet created we insert it
                        if let Some(list_pending_changes) = pending_changes.get_mut(&j) {
                            list_pending_changes.push(sync_event.clone());
                        } else {
                            pending_changes.insert(j, vec![sync_event.clone()]);
                        }
                    }
                }
            }
        }
        // iterate over each gamestate applying all pending changes
        for i in 0..game_state.len() {
            // get the gamestate we are using
            let Some(game_state_a) = game_state.get_mut(i) else {
                println!("Failed to get GameState at index: {}", i);
                continue;
            };

            // get the pending changes. there can be none so this can silently continue
            let Some(pending_changes) = pending_changes.get(&i) else {
                continue;
            };

            // apply all the sync events
            game_state_a.try_apply_network_sync_events(&pending_changes);
        }

        // send events
        for i in 0..event_queue.len() {
            //get queuue
            let event_queue_a = event_queue.get_mut(i).unwrap();
            let events = event_queue_a.try_drain_network_sync_events();

            for event in events {
                match event.ownership {
                    curio_core::collections::event_queue::EventScope::All => {
                        for j in 0..event_queue.len() {
                            // if equal this means that the two queues are the same and we should skipp
                            if i == j {
                                continue;
                            }

                            // get the reciever
                            let event_queue_b = event_queue.get_mut(j).unwrap();

                            // apply to the reciever
                            event_queue_b.try_apply_network_sync_events(vec![event.clone()]);
                        }
                    }
                    curio_core::collections::event_queue::EventScope::ConnectedHost => {
                        for j in 0..event_queue.len() {
                            // if equal this means that the two queues are the same and we should skipp
                            if i == j {
                                continue;
                            }

                            // get reciever
                            let event_queue_b = event_queue.get_mut(j).unwrap();

                            // get network cabailities for reciever
                            let Some(network_capabilities) = &game_state[j].network_capabilities else {
                                continue;
                            };

                            // make sure these have the correct privilage
                            if network_capabilities.privilege != NetworkModes::LocalHost && network_capabilities.privilege != NetworkModes::OnlineHost {
                                continue;
                            }

                            // enqueue
                            event_queue_b.try_apply_network_sync_events(vec![event.clone()]);
                        }
                    }
                    curio_core::collections::event_queue::EventScope::ConnectedPeers => {
                        for j in 0..event_queue.len() {
                            // if equal this means that the two queues are the same and we should skipp
                            // if i == j {
                            //     continue;
                            // }

                            // get the reciever
                            let event_queue_b = event_queue.get_mut(j).unwrap();

                            // get network cabailities for reciever
                            let Some(network_capabilities) = &game_state[j].network_capabilities else {
                                continue;
                            };

                            // make sure these have the correct privilage
                            if network_capabilities.privilege != NetworkModes::LocalPeer && network_capabilities.privilege != NetworkModes::OnlinePeer {
                                continue;
                            }

                            // enqueue
                            event_queue_b.try_apply_network_sync_events(vec![event.clone()]);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    fn set_game_mode(&mut self, game_state: &mut Vec<GameState>, _game_mode: &GameMode) {
        let mut v = vec![];
        for x in game_state.iter() {
            let Some(network_capabilities) = &x.network_capabilities else {
                continue;
            };
            if network_capabilities.privilege == NetworkModes::LocalPeer || network_capabilities.privilege == NetworkModes::OnlinePeer {
                v.push(x.instance_id);
            }
        }
        for gs in game_state.iter_mut() {
            gs.edit::<SysRecordNetwork>(|x| x.set_peer_instance_ids(v.clone()));
        }
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

use serde::{Deserialize, Serialize};

// Serialize any Clone type safely
pub fn to_bytes<T>(value: &T) -> Vec<u8>
where
    T: Clone + Serialize,
{
    bincode::serialize(value).expect("failed to serialize to bytes")
}

pub fn from_bytes<T>(bytes: &[u8]) -> T
where
    T: Clone + for<'de> Deserialize<'de>,
{
    bincode::deserialize(bytes).expect("failed to deserialize from bytes")
}
