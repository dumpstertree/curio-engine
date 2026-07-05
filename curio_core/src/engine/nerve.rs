use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, collections::HashMap, fmt::Display, time::Instant};

use crate::{engine::curio_network::CurioNetwork, AnyQueue, ImpulseCommon, ImpulseNetworkCapabilities, ImpulseSynchronizer};
/// A Nerve is used to transmit Impulses. Impulses can be passed between different nerves by using the Sync event functions.
pub struct Nerve {
    cache: HashMap<i32, AnyQueue>,
    network_capabilities: ImpulseNetworkCapabilities,
    delayed: Vec<(Instant, i32, Box<dyn Any>, Option<ImpulseSynchronizer>)>,
    pub network: CurioNetwork,
}
impl Nerve {
    /// Create a new nerve to transmit impulses based on a Curio Network identity
    pub fn new(network: CurioNetwork) -> Self {
        Nerve {
            cache: HashMap::new(),
            network_capabilities: ImpulseNetworkCapabilities::new(network.me().mode),
            delayed: Vec::new(),
            network,
        }
    }

    /// Update any timed events that are ongoing in this Nerve
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
                if let Some(sync_event) = ownership {
                    self.network_capabilities.enqueue_synchronizer(sync_event);
                }
            } else {
                // Keep events that haven't fired yet
                remaining.push((timer, uid, event, ownership));
            }
        }

        self.delayed = remaining;
    }

    /// Add an event that should be invoked after a delay
    pub fn enqueue_event_delayed<T>(&mut self, event: T, _delay: f32)
    where
        T: Clone + ImpulseCommon + Serialize + DeserializeOwned + Display + 'static,
    {
        // Network Sync Logic
        let mut sync_event = None;
        if self
            .network_capabilities
            .has_write_privilege(event.ownership())
        {
            if let Some(ser) = ImpulseSynchronizer::serialize::<T>(&event) {
                sync_event = Some(ser);
            }
        }

        self.delayed
            .push((Instant::now(), T::id(), Box::new(event), sync_event));
    }

    /// Add an event that will be run at the end of this frame
    pub fn enqueue_event<T>(&mut self, event: T)
    where
        T: Clone + ImpulseCommon + Serialize + DeserializeOwned + Display + 'static,
    {
        // Insert or push into Queue
        self.cache
            .entry(T::id())
            .or_insert_with(AnyQueue::new)
            .push(event.clone());

        if !self
            .network_capabilities
            .has_write_privilege(event.ownership())
        {
            return;
        }

        if let Some(ser) = ImpulseSynchronizer::serialize::<T>(&event) {
            self.network_capabilities.enqueue_synchronizer(ser);
        }
    }

    /// Drain all events that have been enqueued since last drain
    pub fn drain_queued_events<T: 'static>(&mut self) -> Vec<T>
    where
        T: ImpulseCommon + Clone + 'static,
    {
        let key = T::id();
        match self.cache.get_mut(&key) {
            Some(queue) => {
                let drained = queue.drain::<T>();
                drained
            }
            None => Vec::new(),
        }
    }

    /// Try to apply any synchronize event that was pushed from another nerve
    pub fn try_apply_network_sync_events(&mut self, sync_events: Vec<ImpulseSynchronizer>) {
        for sync in sync_events {
            let Some(payload) = sync.deserialize() else {
                println!("failed deserialize");
                continue;
            };

            self.cache
                .entry(sync.impulse_id)
                .or_insert_with(AnyQueue::new)
                .push_boxed(payload);
        }
    }

    /// Try to get any synchronize event that should be pushed to another nerve
    pub fn try_drain_network_sync_events(&mut self) -> Vec<ImpulseSynchronizer> {
        self.network_capabilities.drain_synchronizers()
    }
}
