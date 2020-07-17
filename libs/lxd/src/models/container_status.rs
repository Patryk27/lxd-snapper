use serde::Deserialize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum LxdContainerStatus {
    Aborting,
    Running,
    Starting,
    Stopped,
    Stopping,
}
