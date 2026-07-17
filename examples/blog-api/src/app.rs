use crate::modules::auth::AuthModule;
use crate::modules::blogs::BlogsModule;
use crate::modules::stats::StatsModule;
use crate::modules::tasks::TasksModule;
use crate::welcome::WelcomeModule;
use ironic::metrics::MetricsModule;
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, AuthModule, BlogsModule, StatsModule, TasksModule])]
pub struct AppModule;
