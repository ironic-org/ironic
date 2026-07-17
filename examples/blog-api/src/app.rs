use crate::modules::blogs::BlogsModule;
use crate::modules::stats::StatsModule;
use crate::welcome::WelcomeModule;
use ironic::metrics::MetricsModule;
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, BlogsModule, StatsModule])]
pub struct AppModule;
