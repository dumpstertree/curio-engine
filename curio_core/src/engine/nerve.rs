use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, collections::HashMap, fmt::Display, time::Instant};

use crate::{
    collections::any_queue::AnyQueue,
    engine::{curio::CurioNetwork, igame_event::IGameEvent},
    static_data::global_events::{get_global_event_deserializer, get_global_event_serializer},
    EventNetworkCapabilities, EventSyncEvent,
};

#[derive(Clone, PartialEq)]
pub enum EventScope {
    All,
    Instance,
    ConnectedHost,
    ConnectedPeers,
}

pub struct Nerve {
    // pub name: String,
    cache: HashMap<i32, AnyQueue>,
    network_capabilities: Option<EventNetworkCapabilities>,
    delayed: Vec<(Instant, i32, Box<dyn Any>, Option<EventSyncEvent>)>,
    network: CurioNetwork,
}
impl Nerve {
    pub fn new(network: CurioNetwork) -> Self {
        Nerve {
            // name: String::from(name),
            cache: HashMap::new(),
            network_capabilities: Some(EventNetworkCapabilities::new(network.me().mode)),
            delayed: Vec::new(),
            network,
        }
    }
    pub fn update_timed_events(&mut self) {
        // Take ownership of all delayed events temporarily
        let mut remaining = Vec::with_capacity(self.delayed.len());

        for (timer, uid, event, ownership) in self.delayed.drain(..) {
            if timer.elapsed().as_secs_f32() > 1.0 {
                // Move the Box<dyn Any> here (no borrowing)
                self.cache
                    .entry(uid)
                    .or_insert_with(AnyQueue::new)
                    .push_boxed(event);

                // Network Sync Logic
                if let Some(network_capabilities) = &mut self.network_capabilities {
                    if let Some(sync_event) = ownership {
                        network_capabilities.enqueue_sync_events(sync_event);
                    }
                }
            } else {
                // Keep events that haven't fired yet
                remaining.push((timer, uid, event, ownership));
            }
        }

        self.delayed = remaining;
    }

    pub fn enqueue_event_delayed<T>(&mut self, event: T, _delay: f32)
    where
        T: Clone + IGameEvent + Serialize + DeserializeOwned + Display + 'static,
    {
        // Network Sync Logic
        let Some(network_capabilities) = &mut self.network_capabilities else {
            println!("no network");
            return;
        };

        let mut sync_event = None;
        if network_capabilities.has_write_privilege(event.ownership()) {
            if let Some(ser) = EventSyncEvent::serialize::<T>(&event) {
                sync_event = Some(ser);
            }
        }

        self.delayed
            .push((Instant::now(), T::id(), Box::new(event), sync_event));
    }

    pub fn enqueue_event<T>(&mut self, event: T)
    where
        T: Clone + IGameEvent + Serialize + DeserializeOwned + Display + 'static,
    {
        // Insert or push into Queue
        self.cache
            .entry(T::id())
            .or_insert_with(AnyQueue::new)
            .push(event.clone());

        // Network Sync Logic
        let Some(network_capabilities) = &mut self.network_capabilities else {
            println!("no network");
            return;
        };

        if !network_capabilities.has_write_privilege(event.ownership()) {
            return;
        }

        if let Some(ser) = EventSyncEvent::serialize::<T>(&event) {
            network_capabilities.enqueue_sync_events(ser);
        }
    }

    pub fn drain_queued_events<T: 'static>(&mut self) -> Vec<T>
    where
        T: IGameEvent + Display + Clone + 'static,
    {
        let key = T::id();
        match self.cache.get_mut(&key) {
            Some(queue) => {
                let drained = queue.drain::<T>();
                drained
            }
            None => {
                // println!("no queue found for id {}", key);
                Vec::new()
            }
        }
    }

    pub fn try_apply_network_sync_events(&mut self, sync_events: Vec<EventSyncEvent>) {
        for sync in sync_events {
            let Some(payload) = sync.deserialize() else {
                println!("failed deserialize");
                continue;
            };

            self.cache
                .entry(sync.id)
                .or_insert_with(AnyQueue::new)
                .push_boxed(payload);
        }
    }

    pub fn try_drain_network_sync_events(&mut self) -> Vec<EventSyncEvent> {
        let Some(network_capabilities) = &mut self.network_capabilities else {
            return Vec::new();
        };

        network_capabilities.drain_sync_events()
    }
}

pub trait AsAny {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}
