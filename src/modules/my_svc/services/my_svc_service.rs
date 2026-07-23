use ironic::prelude::*;

#[derive(Injectable)]
pub struct MySvcService;

impl MySvcService {
    #[must_use]
    #[allow(dead_code)]
    pub const fn name(&self) -> &'static str {
        "my-svc"
    }
}
