use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MenuItem {
    pub id: u32,
    pub desc: String,
}
