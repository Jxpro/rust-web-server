use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;
type Receiver = Arc<Mutex<mpsc::Receiver<Job>>>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

#[derive(Debug)]
pub enum ThreadPoolError {
    ZeroSizedPool,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        Self::build(size).unwrap()
    }

    /// Build a ThreadPool with Result.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Errors
    ///
    /// The `build` function will return an error if the size is zero.
    pub fn build(size: usize) -> Result<ThreadPool, ThreadPoolError> {
        if size == 0 {
            return Err(ThreadPoolError::ZeroSizedPool);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
            println!("Worker {} created.", id);
        }

        Ok(ThreadPool { workers, sender })
    }

    /// Execute a job in the ThreadPool.
    ///
    /// The job is a closure that implements `FnOnce() + Send + 'static`.
    ///
    /// # Panics
    ///
    /// The `execute` function will panic if the sender is disconnected.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

impl Worker {
    pub fn new(id: usize, receiver: Receiver) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {id} got a job; executing.");
            job();
        });
        Self { id, thread }
    }
}
