use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct BlogPost {
    pub name: String,
    pub body: String
}

