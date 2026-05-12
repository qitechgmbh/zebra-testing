use std::time::Duration;

use crate::stream::ExitCondition;
use super::types::{
    NextState, 
    StateTransition,
    Stream, 
    Result
};

pub async fn run(mut stream: Stream) -> Result {
    let timeout = Duration::from_secs(2);
    let exit_condition = ExitCondition::ShutdownOrTimer(timeout);

    let line = stream.read_line(exit_condition).await?;
    let name = line.trim_end().to_string();

    Ok(StateTransition {
        next: NextState::RecvData(name),
        stream,
    })
}