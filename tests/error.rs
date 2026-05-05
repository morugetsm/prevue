use prevue::{Error, render};
use serde::{Serialize, Serializer};

struct BrokenData;

impl Serialize for BrokenData {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom("broken data"))
    }
}

#[test]
fn data_serialization_error_returns_error() {
    let err = render("<p>{{ message }}</p>", BrokenData).unwrap_err();
    assert!(matches!(err, Error::DataSerialize { .. }));
}
