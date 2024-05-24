use rand::seq::SliceRandom;
use rand::Rng;

pub struct ReplayBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
}

impl<T> ReplayBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        ReplayBuffer {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, experience: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(experience);
        } else {
            let index = rand::thread_rng().gen_range(0..self.capacity);
            self.buffer[index] = experience;
        }
    }

    pub fn sample(&self, sample_size: usize) -> Vec<&T> {
        let mut rng = rand::thread_rng();
        let sample = self.buffer.choose_multiple(&mut rng, sample_size).collect();
        sample
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.buffer.len() == self.capacity
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/*
This module implements a replay buffer data structure (used in the reinforcement learning algorithms). It stores a fixed-size buffer of experiences, and when the buffer is full, new experiences are randomly added to replace older ones.

The ReplayBuffer struct is generic and can be used to store any type of experience. It has a capacity parameter that specifies the maximum number of experiences that can be stored in the buffer.

The add method is used to add a new experience to the buffer. If the buffer is not yet full, the experience is simply appended to the end of the buffer. Otherwise, a random experience in the buffer is replaced with the new one.

The sample method is used to retrieve a random subset of experiences from the buffer. It takes a sample_size parameter that specifies the number of experiences to sample. It uses the rand::thread_rng() function to generate random numbers for selecting experiences.

The len method returns the current number of experiences in the buffer.

The is_full method returns a boolean indicating whether the buffer is full or not.

The clear method is used to clear the contents of the buffer.
 */