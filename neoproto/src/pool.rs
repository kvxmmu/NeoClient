use {
    crate::{
        serializer::ClientId
    },

    std::{
        collections::BTreeSet
    }
};

pub struct IDPool {
    pub deq: BTreeSet<ClientId>,
    pub last: ClientId,
}

impl IDPool {
    pub fn request_id(&mut self) -> ClientId {
        if self.deq.is_empty() {
            let was = self.last.clone();
            self.last += 1;

            was
        } else {
            let value = *self.deq.iter().next().unwrap();
            self.deq.remove(&value);

            value
        }
    }

    pub fn return_id(&mut self, id: ClientId) {
        self.deq.insert(id);
    }

    pub fn new(start: ClientId) -> IDPool {
        IDPool {
            last: start,
            deq: Default::default(),
        }
    }
}
