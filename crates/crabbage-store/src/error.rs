use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A failure to open the database or bring its schema up to date.
#[derive(Debug)]
pub enum StoreError {
    /// The data directory could not be created.
    Io(std::io::Error),
    /// The database could not be opened or queried.
    Database(sqlx::Error),
    /// The embedded migrations could not be applied.
    Migrate(sqlx::migrate::MigrateError),
    /// A name or alias is already taken, or would equal a name or alias of another ingredient. The
    /// text is the name or alias that clashes.
    Conflict(String),
    /// The ingredient is used by recipe lines and cannot be deleted.
    InUse,
    /// A step links to a line position that the recipe does not have.
    UnknownLine,
    /// A rating is not from 1 to 5 stars.
    InvalidStars,
    /// The text is blank after trimming whitespace.
    Empty,
    /// A planner date is not a valid calendar date in `YYYY-MM-DD` form.
    InvalidDate,
    /// Planner settings or entry servings are outside their allowed range.
    InvalidPlannerSettings,
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "cannot create the data directory: {error}"),
            Self::Database(error) => write!(f, "cannot open the database: {error}"),
            Self::Migrate(error) => write!(f, "cannot run the migrations: {error}"),
            Self::Conflict(text) => write!(f, "`{text}` is already a name or alias"),
            Self::InUse => write!(f, "the ingredient is used by recipes"),
            Self::UnknownLine => write!(f, "a step links to a line the recipe does not have"),
            Self::InvalidStars => write!(f, "a rating must be from 1 to 5 stars"),
            Self::Empty => write!(f, "the text is empty"),
            Self::InvalidDate => write!(f, "the date must be a valid YYYY-MM-DD date"),
            Self::InvalidPlannerSettings => write!(f, "planner settings are invalid"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Database(error) => Some(error),
            Self::Migrate(error) => Some(error),
            Self::Conflict(_)
            | Self::InUse
            | Self::UnknownLine
            | Self::InvalidStars
            | Self::Empty
            | Self::InvalidDate
            | Self::InvalidPlannerSettings => None,
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<sqlx::migrate::MigrateError> for StoreError {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(error)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use sqlx::migrate::MigrateError;

    use super::StoreError;

    #[test]
    fn each_variant_names_its_failure_and_keeps_its_source() {
        let errors = [
            StoreError::from(std::io::Error::other("denied")),
            StoreError::from(sqlx::Error::PoolTimedOut),
            StoreError::from(MigrateError::VersionMissing(1)),
        ];
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert!(messages[0].starts_with("cannot create the data directory: denied"));
        assert!(messages[1].starts_with("cannot open the database: "));
        assert!(messages[2].starts_with("cannot run the migrations: "));
        assert!(errors.iter().all(|error| error.source().is_some()));
    }
}
