use ironic::prelude::*;
use crate::welcome::WelcomeModule;
use crate::modules::todos::TodosModule;
use ironic::metrics::MetricsModule;

#[derive(Module)]
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, TodosModule])]
pub struct AppModule;
