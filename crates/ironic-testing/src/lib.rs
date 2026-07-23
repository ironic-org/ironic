#![doc = include_str!("../README.md")]

mod application;
mod error;
mod module;
mod request;
mod response;

pub use application::{TestApplication, TestApplicationBuilder};
pub use error::TestBuildError;
pub use module::{CompiledTestModule, TestModule, TestModuleBuilder};
pub use request::TestRequestBuilder;
pub use response::TestResponse;

#[cfg(test)]
mod tests {
    /// Compile-time check that all re-exported types are accessible.
    #[allow(dead_code)]
    fn _re_exported_types_accessible() {
        // These compile — if any type is missing, the module won't compile.
        fn check<T>() {}
        check::<crate::testing::TestBuildError>();
        check::<crate::testing::TestApplication>();
        check::<crate::testing::TestApplicationBuilder>();
        check::<crate::testing::CompiledTestModule>();
        check::<crate::testing::TestModule>();
        check::<crate::testing::TestModuleBuilder>();
        check::<crate::testing::TestResponse>();
    }
}
