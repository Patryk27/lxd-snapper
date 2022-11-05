use serde::Deserialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum LxdInstanceStatus {
    Aborting,
    Running,
    Starting,
    Stopped,
    Stopping,
    Ready,
}
