use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct ATProtoServerCreateSession {
    pub(crate) identifier: String,
    pub(crate) password: String,
}
