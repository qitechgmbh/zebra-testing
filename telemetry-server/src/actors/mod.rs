use std::sync::Arc;

use actix::prelude::*;

use crate::config::Config;

struct Ingest {
    pub config: Arc<Config>
}

impl Actor for Ingest {
    type Context = Context<Self>;
}

impl Handler<Ping> for Ingest {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
        self.count += msg.0;

        self.count
    }
}

#[derive(Message)]
#[rtype(result = "usize")]
struct Ping(usize);