//! Recipe completion: the deterministic reading of a web draft, the AI pass when configured, and
//! the merge of the AI's draft with the reading.

mod deterministic;
mod imported;
mod merge;
mod photos;
mod run;

pub use deterministic::from_draft;
pub use imported::{Imported, ImportedLine, ImportedStep};
pub use merge::merge;
pub use photos::from_photos;
pub use run::complete;
