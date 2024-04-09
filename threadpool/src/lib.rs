use core::fmt;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct Threadpool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Threadpool {
    /// Create a new Threadpool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// The 'build' function will return an PoolCreationError if the size is zero.
    /// ```
    /// use threadpool::Threadpool;
    /// let pool = Threadpool::build(4);
    /// ```
    pub fn build(size: usize) -> Result<Threadpool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError);
        } else {
            let (sender, reciever) = mpsc::channel();

            let reciever = Arc::new(Mutex::new(reciever));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&reciever)));
            }
            return Ok(Threadpool {
                workers,
                sender: Some(sender),
            });
        }
    }

    /// Execute a closure using a thread from the pool.
    ///
    /// ```
    /// use threadpool::Threadpool;
    /// let pool = Threadpool::build(1).unwrap();
    ///
    /// pool.execute(|| {println!("executing...")})
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            };
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = reciever.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, Clone, PartialEq)]
pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid size value provided to Threadpool::build")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_size_returns_err() {
        let result = Threadpool::build(0);

        // let expected = Err(PoolCreationError);

        assert!(result.is_err())
    }
}
