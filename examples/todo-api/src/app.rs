use ironic::prelude::*;
#[derive(Module)]
#[module(
    imports = [crate::modules::todos::TodosModule],
    providers = [],
    controllers = [],
    exports = [],
)]
pub struct AppModule;
