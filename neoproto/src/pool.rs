use {
    crate::serializer::ClientId,
    std::collections::VecDeque
};

pub struct IDPool {
    pub deq: VecDeque<ClientId>,
    pub last: ClientId
}

impl IDPool {
    pub fn request_id(&mut self) -> ClientId {
        if self.deq.is_empty() {
            let was = self.last.clone();
            self.last += 1;

            was
        } else {
            self.deq.pop_back().unwrap()
        }
    }

    pub fn return_id(&mut self, id: ClientId) {
        self.deq.push_back(id);
    }

    pub fn new(start: ClientId) -> IDPool {
        IDPool{last: start, deq: Default::default()}
    }
}


