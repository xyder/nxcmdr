
pub type BoxedResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
