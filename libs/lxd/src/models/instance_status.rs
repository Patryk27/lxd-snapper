use serde::Deserialize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum LxdInstanceStatus {
    Aborting,
    Running,
    Starting,
    Stopped,
    Stopping,
    Ready,
}
