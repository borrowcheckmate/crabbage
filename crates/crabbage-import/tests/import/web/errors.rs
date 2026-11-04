use crabbage_import::web::ImportError;

#[test]
fn every_error_has_a_clear_message() {
    let messages = [
        (
            ImportError::NotHttp,
            "only http and https addresses can be imported",
        ),
        (
            ImportError::PrivateAddress,
            "the address is not a public internet address",
        ),
        (ImportError::Timeout, "the page took too long to answer"),
        (
            ImportError::TooLarge,
            "the page is larger than the import limit",
        ),
        (
            ImportError::Status(404),
            "the page answered with status 404",
        ),
        (
            ImportError::Network("connection reset".to_owned()),
            "the page could not be fetched: connection reset",
        ),
        (
            ImportError::NoRecipe,
            "the page has no schema.org Recipe data",
        ),
    ];
    for (error, text) in messages {
        assert_eq!(error.to_string(), text);
    }
}
