use std::pin::Pin;

use tokio::sync::mpsc::Sender;




pub type ServiceId = u32;
pub type RunnerId  = u64;

pub trait Launcher {
    type Fut: Future<Output = ()> + Send;
    fn launch(&self, sender: RunnerSender) -> Self::Fut;
}

pub struct RunnerSender {
    service_id: ServiceId,
    tx: Sender<Box<dyn Runner>>
}

impl RunnerSender {
    pub async fn send(&self, runner: Box<dyn Runner>) {
        let signal = OverseerSignal::DistributeMessage(self.service_id, message);
        self.tx.send(signal).await
            .expect("If channel to overseer is destroyed then we are fucked");
    }
}

pub trait Runner {
    fn run(self, sender: MessageSender) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub struct MessageSender {
    service_id: ServiceId,
    tx: Sender<OverseerSignal>
}

impl MessageSender {
    pub async fn send(&self, message: Vec<u8>) {
        let signal = OverseerSignal::DistributeMessage(self.service_id, message);
        self.tx.send(signal).await
            .expect("If channel to overseer is destroyed then we are fucked");
    }
}

pub struct Overseer {
    services: ServiceEntry,
}

pub enum OverseerSignal {
    Shutdown, 
    RestartLauncher(ServiceId, u32),
    SpawnRunner(ServiceId),
    DistributeMessage(ServiceId, Vec<u8>),
}

pub struct ServiceEntry {

}

// Send message DOWN
// Runner after each cycle may leave a message
// 
