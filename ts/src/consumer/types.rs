use crate::types::ClientTask;

#[derive(Debug)]
pub enum State {
    RecvRequest(ClientTask<Result<Route, RecvRequestError>>),
    SendRecvError(RecvRequestError),
    StreamLive(ClientTask<()>),
    SendQuery(ClientTask<()>),
    SendStatus(ClientTask<()>),
    SendExit(ClientTask<()>)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RecvRequestError {
    HttpParse(httparse::Error),
    MethodMissingg,
    MethodUnsupported,
    PathMissing,
    UnknownRoute,
}

#[derive(Debug, Clone)]
pub enum Route {
    Machine(MachineRoute),
    System(SystemRoute),
}

#[derive(Debug, Clone)]
pub enum MachineRoute {
    Live { 
        name: String,
    },
    Query {
        name:  String,
        table: String,
        query: Option<String>,
    }
}

#[derive(Debug, Clone)]
pub enum SystemRoute {
    Status,
}