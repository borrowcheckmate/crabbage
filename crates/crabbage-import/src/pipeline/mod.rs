//! Recipe completion: the reading of a web draft without AI, the AI pass when the draft needs it,
//! and the merge of the AI's draft with the reading.

mod decide;
mod deterministic;
mod imported;
mod merge;
mod photos;
mod run;

pub use decide::needs_ai;
pub use deterministic::from_draft;
pub use imported::{Imported, ImportedLine, ImportedStep};
pub use merge::merge;
pub use photos::from_photos;
pub use run::complete;
