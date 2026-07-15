use crate::modules::todos::TodosModule;
use crate::welcome::WelcomeModule;
use ironic::metrics::MetricsModule;
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, TodosModule])]
pub struct AppModule;
