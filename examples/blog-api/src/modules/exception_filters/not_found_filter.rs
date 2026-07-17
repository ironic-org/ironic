use ironic::{ExceptionFilter, FilterContext, FrameworkResponse, HttpError, HttpStatus};

pub struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if error.status() == HttpStatus::NOT_FOUND {
            let body = ironic::json::json!({
                "error": error.code(),
                "message": error.message(),
                "status": 404,
            });
            Ok(FrameworkResponse::json(HttpStatus::NOT_FOUND, &body)?)
        } else {
            Err(error.clone())
        }
    }
}
