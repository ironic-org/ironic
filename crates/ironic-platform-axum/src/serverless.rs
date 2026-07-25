//! AWS Lambda adapter for Ironic applications.
//!
//! Wraps a compiled [`AxumApplication`] as a Lambda handler.
//! Requires the `serverless` feature.

use crate::AxumApplication;

impl AxumApplication {
    /// Converts this application into a Lambda-compatible handler and
    /// runs it. This function never returns under normal Lambda operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the Lambda runtime fails to initialize.
    pub async fn run_lambda(self) -> Result<(), lambda_http::Error> {
        let router = self.into_router();
        lambda_http::run(router).await
    }
}

/// A type alias for Lambda-compatible function errors.
pub type LambdaError = lambda_http::Error;
