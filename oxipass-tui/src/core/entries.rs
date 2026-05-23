use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Entry {
    Login {
        id: Uuid,
        name: String,
        username: Option<String>,
        email: Option<String>,
        password: String,
        url: Option<String>,
    },
    Payment {
        id: Uuid,
        name: String,
        cardholder: String,
        card_number: String,
        exp_date: String,
        cvv: String,
    },
    Note {
        id: Uuid,
        name: String,
        description: Option<String>,
        content: String,
    },
}
