pub mod cors;
pub mod error;
pub mod headers;
pub mod query;
pub mod registrar;
pub mod resource_id;
pub mod same_project;
pub mod slug;

pub fn map_u32(signed: Option<i64>) -> Result<Option<u32>, std::num::TryFromIntError> {
    Ok(if let Some(signed) = signed {
        Some(signed.try_into()?)
    } else {
        None
    })
}
