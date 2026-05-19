use std::any::Any;

pub struct AnyQueue {
    items: Vec<Box<dyn Any>>,
}

impl AnyQueue {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Push a typed event into the queue
    pub fn push<T: 'static>(&mut self, event: T) {
        self.items.push(Box::new(event));
    }
    pub fn push_boxed(&mut self, event: Box<dyn Any>) {
        self.items.push(event);
    }

    /// Drain all events of type T, removing them from the queue,
    /// returning owned T values.
    pub fn drain<T: 'static>(&mut self) -> Vec<T> {
        let mut out = Vec::new();
        let mut remaining = Vec::new();

        for boxed in self.items.drain(..) {
            // Try to downcast and keep only matching items
            match boxed.downcast::<T>() {
                Ok(boxed_t) => out.push(*boxed_t),
                Err(other) => remaining.push(other),
            }
        }

        self.items = remaining;
        out
    }

    /// Get a read-only view of all events of type T (without draining)
    pub fn get_all<T: 'static>(&self) -> Vec<&T> {
        self.items
            .iter()
            .filter_map(|item| item.downcast_ref::<T>())
            .collect()
    }

    /// Returns whether any item of type T is in the queue
    pub fn has_type<T: 'static>(&self) -> bool {
        self.items.iter().any(|item| item.is::<T>())
    }

    /// Return total items in queue (all types)
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
