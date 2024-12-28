use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;
type Receiver = Arc<Mutex<mpsc::Receiver<Job>>>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
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

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
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
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Worker {
    pub fn new(id: usize, receiver: Receiver) -> Self {
        let thread = thread::spawn(move || loop {
            // 如下代码可以编译运行，但是打不到异步的效果。
            // 因为 if let 右边的临时值 MutexGuard<T>，其生命周期存在于后面的代码块中，
            // 这会导致 job 运行结束之前一直持有 receiver 的锁，导致其他线程无法获取锁，
            // 但是 let 语句会丢弃临时值，所以在 job 运行之前就释放了锁。
            // if let Ok(job) = receiver.lock().unwrap().recv() {
            //     println!("Worker {id} got a job; executing.");
            //     job();
            // } else {
            //     println!("Worker {id} disconnected; shutting down.");
            //     return;
            // }
            let job = receiver.lock().unwrap().recv();
            match job {
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
        Self {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {id}.", id = worker.id);
            worker.thread.take().unwrap().join().unwrap();
        }
    }
}
