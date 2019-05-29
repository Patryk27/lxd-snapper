use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum LxdContainerStatus {
    Aborting,
    Running,
    Starting,
    Stopped,
    Stopping,
}