use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub ip_address: Option<String>,
    pub status: CallStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Idle,
    Calling,
    InCall,
    OnHold,
    Offline,
}

impl Default for CallStatus {
    fn default() -> Self {
        CallStatus::Offline
    }
}

impl User {
    pub fn new(id: String, username: String) -> Self {
        User {
            id,
            username,
            ip_address: None,
            status: CallStatus::Idle,
        }
    }

    pub fn set_ip_address(&mut self, ip: String) {
        self.ip_address = Some(ip);
    }

    pub fn set_status(&mut self, status: CallStatus) {
        self.status = status;
    }
}
