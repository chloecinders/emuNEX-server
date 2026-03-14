mod api_error;
pub use api_error::V1ApiError;

mod api_response;
pub use api_response::V1ApiResponse;

pub type V1ApiResponseType<T> = Result<V1ApiResponse<T>, V1ApiError>;

pub mod v1;
