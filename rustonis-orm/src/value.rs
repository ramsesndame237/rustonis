/// Type-safe binding value for QueryBuilder conditions.
///
/// Covers the most common SQL column types. Use `OrmValue::from(val)` for
/// ergonomic construction.
#[derive(Debug, Clone)]
pub enum OrmValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    Null,
}

impl From<i32>    for OrmValue { fn from(v: i32)    -> Self { OrmValue::Int(v as i64) } }
impl From<i64>    for OrmValue { fn from(v: i64)    -> Self { OrmValue::Int(v) } }
impl From<u32>    for OrmValue { fn from(v: u32)    -> Self { OrmValue::Int(v as i64) } }
impl From<f32>    for OrmValue { fn from(v: f32)    -> Self { OrmValue::Float(v as f64) } }
impl From<f64>    for OrmValue { fn from(v: f64)    -> Self { OrmValue::Float(v) } }
impl From<bool>   for OrmValue { fn from(v: bool)   -> Self { OrmValue::Bool(v) } }
impl From<&str>   for OrmValue { fn from(v: &str)   -> Self { OrmValue::Text(v.to_string()) } }
impl From<String> for OrmValue { fn from(v: String) -> Self { OrmValue::Text(v) } }
impl<T: Into<OrmValue>> From<Option<T>> for OrmValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None    => OrmValue::Null,
        }
    }
}
