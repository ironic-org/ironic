use std::time::Instant;

use ironic::{Interceptor, InterceptorNext, PipelineFuture, RequestContext};

pub struct TimingInterceptor;

impl Interceptor for TimingInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let path = context.request().uri().path().to_owned();
            let method = context.request().method().clone();
            let start = Instant::now();

            let response = next.run(context).await?;

            let elapsed = start.elapsed();
            let status = response.status().as_u16();
            tracing::info!(
                target: "ironic.http.timing",
                http_method = %method,
                http_path = %path,
                http_status = status,
                duration_ms = (elapsed.as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            );

            Ok(response)
        })
    }
}
