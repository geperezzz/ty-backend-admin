use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_with::rust::double_option;

#[derive(Deserialize)]
pub struct MaybeNull<T: DeserializeOwned>(
    #[serde(deserialize_with = "double_option::deserialize")]
    Option<Option<T>>
);

impl<T: DeserializeOwned> From<Option<T>> for MaybeNull<T> {
    fn from(value: Option<T>) -> MaybeNull<T> {
        MaybeNull(Some(value))
    }
}

impl<T: DeserializeOwned> From<MaybeNull<T>> for Option<T> {
    fn from(value: MaybeNull<T>) -> Option<T> {
        value.0.unwrap()
    }
}

fn deserialize_as_inner<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
D: Deserializer<'de>,
T: Deserialize<'de>,
{
    Ok(Some(T::deserialize(deserializer)?))
}

#[derive(Deserialize)]
pub struct MaybeAbsent<T: DeserializeOwned>(
    #[serde(deserialize_with = "deserialize_as_inner")]
    Option<T>
);

impl<T: DeserializeOwned> Default for MaybeAbsent<T> {
    fn default() -> MaybeAbsent<T> {
        MaybeAbsent(None)
    }
}

impl<T: DeserializeOwned> From<Option<T>> for MaybeAbsent<T> {
    fn from(value: Option<T>) -> MaybeAbsent<T> {
        MaybeAbsent(value)
    }
}

impl<T: DeserializeOwned> From<MaybeAbsent<T>> for Option<T> {
    fn from(value: MaybeAbsent<T>) -> Option<T> {
        value.0
    }
}

impl<T: DeserializeOwned> From<MaybeAbsent<MaybeNull<T>>> for Option<Option<T>> {
    fn from(value: MaybeAbsent<MaybeNull<T>>) -> Option<Option<T>> {
        let optional_nullable: Option<MaybeNull<T>> = value.into();
        optional_nullable.map(MaybeNull::into)
    }
}