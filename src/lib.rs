pub mod cli;
mod config;
mod req;
mod utils;

pub use config::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};
pub use utils::{diff_text, highlight_text};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
