use crate::modules::example::ExampleModule;
use crate::welcome::WelcomeModule;
use ironic::metrics::MetricsModule;
use ironic::prelude::*;
#[derive(Module)]
#[module(
    imports = [HealthModule,
    MetricsModule,
    WelcomeModule,
    ExampleModule,
    crate::modules::auth::AuthModule],
    providers = [],
    controllers = [],
    exports = [],
)]
pub struct AppModule;
