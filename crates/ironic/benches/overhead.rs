//! Lightweight, dependency-free comparative runtime benchmark.

use std::{hint::black_box, sync::Arc, time::Instant};

use axum::{Router, body::Body, http::Request, routing::get};
use ironic::{
    ContainerBuilder, TestApplication, build_http_application, compile_controller_routes,
    compile_module_graph, prelude::*,
};
use tower::ServiceExt;

struct BenchController;

fn controller() -> ControllerDefinition {
    let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
        Ok(BenchController)
    });
    let route = RouteDefinition::new(
        HttpMethod::GET,
        "/",
        "index",
        handler_fn(|_controller: Arc<BenchController>, _arguments| async {
            Ok::<_, HttpError>("ok")
        }),
    )
    .expect("benchmark route is valid");
    ControllerDefinition::new::<BenchController>("/bench", provider)
        .expect("benchmark controller is valid")
        .route(route)
}

struct BenchModule;

impl Module for BenchModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .controller(controller())
            .build()
    }
}

struct TransientProvider;

fn main() {
    const METADATA_ITERATIONS: u64 = 10_000;
    const REQUEST_ITERATIONS: u64 = 2_000;

    measure("module graph compilation", METADATA_ITERATIONS, || {
        black_box(compile_module_graph(BenchModule::definition()).unwrap());
    });
    measure("route registration", METADATA_ITERATIONS, || {
        black_box(compile_controller_routes([controller()]).unwrap());
    });

    let mut container = ContainerBuilder::new();
    container
        .register(ProviderDefinition::constructor(
            Scope::Transient,
            Vec::new(),
            |_resolver| Ok(TransientProvider),
        ))
        .unwrap();
    let container = container.build();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let started = Instant::now();
    runtime.block_on(async {
        for _ in 0..METADATA_ITERATIONS {
            black_box(container.resolve::<TransientProvider>().await.unwrap());
        }
    });
    report(
        "transient provider resolution",
        METADATA_ITERATIONS,
        started,
    );

    let graph = compile_module_graph(BenchModule::definition()).unwrap();
    measure("HTTP runtime startup", METADATA_ITERATIONS, || {
        black_box(build_http_application(&graph).unwrap());
    });

    runtime.block_on(async {
        let application = TestApplication::new::<BenchModule>().await.unwrap();
        let started = Instant::now();
        for _ in 0..REQUEST_ITERATIONS {
            black_box(application.get("/bench").send().await);
        }
        report("Ironic in-process request", REQUEST_ITERATIONS, started);
        application.shutdown().await.unwrap();

        let raw = Router::new().route("/bench", get(|| async { "ok" }));
        let started = Instant::now();
        for _ in 0..REQUEST_ITERATIONS {
            black_box(
                raw.clone()
                    .oneshot(Request::get("/bench").body(Body::empty()).unwrap())
                    .await
                    .unwrap(),
            );
        }
        report("raw Axum in-process request", REQUEST_ITERATIONS, started);
    });
}

fn measure(label: &str, iterations: u64, mut operation: impl FnMut()) {
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    report(label, iterations, started);
}

fn report(label: &str, iterations: u64, started: Instant) {
    let elapsed = started.elapsed();
    let nanos = elapsed.as_nanos() / u128::from(iterations);
    println!("{label:<32} {nanos:>10} ns/op ({iterations} iterations)");
}
