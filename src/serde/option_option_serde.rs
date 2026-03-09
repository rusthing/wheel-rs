use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(d).map(Some)
    // key 存在值为 null  → Some(None)
    // key 存在有值       → Some(Some(v))
    // key 不存在         → 走 Default::default() → None
}
