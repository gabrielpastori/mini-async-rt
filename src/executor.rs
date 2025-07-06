use std::{sync::{mpsc, Arc, Mutex}, task::Context};

use crate::task::Task;

pub struct Executor {
    ready_queue: mpsc::Receiver<Arc<Task>>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future = task.future.lock().unwrap();

            // make a context (explained later)
            let waker = Arc::clone(&task).waker();
            let mut context = Context::from_waker(&waker);

            // Allow the future some CPU time to make progress
            let _ = future.as_mut().poll(&mut context);
        }
    }
}


#[derive(Clone)]
pub struct Spawner {
    task_sender: mpsc::SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            spawner: self.clone(),
        });

        self.spawn_task(task)
    }

    pub(crate) fn spawn_task(&self, task: Arc<Task>) {
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

pub fn new_executor_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;

    let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);

    (Executor { ready_queue }, Spawner { task_sender })
}