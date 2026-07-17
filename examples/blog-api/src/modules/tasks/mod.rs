use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::blogs::services::BlogService;

#[derive(Injectable)]
pub struct StatsReporter {
    service: Arc<BlogService>,
}

impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> ironic::LifecycleFuture<'_> {
        let svc = Arc::clone(&self.service);
        Box::pin(async move {
            let _task = ironic::services::scheduling::cron("0 * * * * *", move || {
                let svc = Arc::clone(&svc);
                async move {
                    match svc.stats() {
                        Ok(s) => {
                            ironic::logging::log::info!(
                                total = s.total,
                                published = s.published,
                                "hourly blog stats (cron)"
                            );
                        }
                        Err(e) => {
                            ironic::logging::log::error!(error = %e, "failed to collect stats");
                        }
                    }
                }
            });

            ironic::logging::log::info!("hourly stats cron reporter started");
            Ok(())
        })
    }
}

#[derive(Module)]
#[module(
    imports = [crate::modules::blogs::BlogsModule],
    providers = [StatsReporter],
)]
pub struct TasksModule;
