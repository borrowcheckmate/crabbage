use std::path::PathBuf;

use super::step::Step;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    Run(Step),
    Policy,
    Identity(Option<String>),
    CommitMsg(PathBuf),
    PrCheck,
}
