use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

struct Overseer<T> {
    tasks: FuturesUnordered<JoinHandle<T>>,
}

impl<T> Overseer<T> {
    fn new() -> Self {
        Self { tasks: FuturesUnordered::new() }
    }

    fn add_task(&mut self, task: JoinHandle<T>) {
        self.tasks.push(task);
    }

    async fn next_completed(&mut self) -> Option<T> {
        // Wait for any task to finish
        self.tasks.next().await.map(|res| res.expect("Task panicked"))
    }
}

pub async fn run_overseer() {
    let tasks = FuturesUnordered::<JoinHandle<()>>::new();

    
}