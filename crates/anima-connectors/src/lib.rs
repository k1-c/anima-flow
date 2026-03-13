pub mod traits;

pub mod calendar;
pub mod chatwork;
pub mod gmail;
pub mod linear;
pub mod slack;
pub mod todoist;

pub use traits::{
    Connector, ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest,
    MutationResult,
};
