use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetfunnelStatus {
    Pass,
    Wait,
    AlreadyCompleted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetfunnelSnapshot {
    pub status: NetfunnelStatus,
    pub waiting_count: Option<u32>,
}
