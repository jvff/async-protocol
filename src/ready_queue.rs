use std::collections::VecDeque;

pub struct ReadyQueue<T> {
    queue: VecDeque<Option<T>>,
    first_id: usize,
}

impl<T> ReadyQueue<T> {
    pub fn new() -> Self {
        ReadyQueue {
            queue: VecDeque::new(),
            first_id: 0,
        }
    }

    pub fn push(&mut self, item: T) -> usize {
        self.queue.push_back(Some(item));

        self.first_id + self.queue.len()
    }

    pub fn pop(&mut self, id: usize) -> T {
        if id < self.first_id {
            panic!("item popped twice");
        }

        let position = id - self.first_id;

        if position == 0 {
            let item = self.queue
                .pop_front()
                .expect("no items in queue")
                .expect("item popped twice");

            self.first_id += 1;

            let non_empty_slot = self.queue.iter().position(Option::is_some);

            if let Some(first_non_empty_slot) = non_empty_slot {
                self.queue.drain(0..first_non_empty_slot);
                self.first_id += first_non_empty_slot;
            }

            item
        } else {
            self.queue[position].take().expect("item popped twice")
        }
    }
}
