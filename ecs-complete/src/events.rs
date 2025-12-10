use std::marker::PhantomData;

pub struct Events<T> {
    events: Vec<T>,
    start_index: usize,
}

impl<T> Events<T> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_index: 0,
        }
    }

    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.start_index = 0;
    }

    pub fn update(&mut self) {
        // Mark current events as "old" - they'll be available until next update
        self.start_index = self.events.len();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events[self.start_index..].iter()
    }

    pub fn len(&self) -> usize {
        self.events.len() - self.start_index
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventReader<'a, T> {
    events: &'a Events<T>,
    last_read: usize,
}

impl<'a, T> EventReader<'a, T> {
    pub fn new(events: &'a Events<T>) -> Self {
        Self {
            events,
            last_read: events.start_index,
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &'a T> {
        let start = self.last_read;
        self.last_read = self.events.events.len();
        self.events.events[start..self.last_read].iter()
    }

    pub fn len(&self) -> usize {
        self.events.events.len() - self.last_read
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct EventWriter<'a, T> {
    events: &'a mut Events<T>,
    _marker: PhantomData<T>,
}

impl<'a, T> EventWriter<'a, T> {
    pub fn new(events: &'a mut Events<T>) -> Self {
        Self {
            events,
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }
}
