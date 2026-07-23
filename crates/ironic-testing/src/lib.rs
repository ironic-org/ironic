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
        fn _check<T>() {}
        _check::<crate::testing::TestBuildError>();
        _check::<crate::testing::TestApplication>();
        _check::<crate::testing::TestApplicationBuilder>();
        _check::<crate::testing::CompiledTestModule>();
        _check::<crate::testing::TestModule>();
        _check::<crate::testing::TestModuleBuilder>();
        _check::<crate::testing::TestResponse>();
    }
}
