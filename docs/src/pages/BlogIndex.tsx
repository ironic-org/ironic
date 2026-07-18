import { ArrowRight, Calendar, Clock, GitBranch, Layers, Sparkles, Star } from 'lucide-react';
import { Link } from 'react-router-dom';
import { LATEST_VERSION_LABEL } from '@/lib/constants';

type Post = {
    slug: string;
    title: string;
    description: string;
    date: string;
    tag: 'release' | 'feature' | 'fix' | 'major' | 'deep-dive';
    readTime: string;
};

const posts: Post[] = [
    {
        slug: 'v1.0.7',
        title: 'v1.0.7 — add pagination extractor and SQL error mapping utilities',
        description: 'add pagination extractor and SQL error mapping utilities',
        date: '2026-07-18',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.6',
        title: 'v1.0.6 — implement blog API example with CRUD operations and JWT authentication',
        description: 'implement blog API example with CRUD operations and JWT authentication',
        date: '2026-07-18',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.5',
        title: 'v1.0.5 — implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks',
        description: 'implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks',
        date: '2026-07-18',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.4',
        title: 'v1.0.4 — add async test macro to simplify testing without external dependencies',
        description: 'add async test macro to simplify testing without external dependencies',
        date: '2026-07-17',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.3',
        title: 'v1.0.3 — add async test macro to simplify testing without external dependencies',
        description: 'add async test macro to simplify testing without external dependencies',
        date: '2026-07-17',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.2',
        title: 'v1.0.2 — enhance release workflow with version detection and conditional execution',
        description: 'enhance release workflow with version detection and conditional execution',
        date: '2026-07-17',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.1',
        title: 'v1.0.1 — single version source of truth in docs/lib/constants.ts',
        description: 'single version source of truth in docs/lib/constants.ts',
        date: '2026-07-17',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v1.0.0',
        title: 'v1.0.0 — release v1.0.0 with CI/CD improvements, matrix testing, and blog API example',
        description: 'release v1.0.0 with CI/CD improvements, matrix testing, and blog API example',
        date: '2026-07-17',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.5.0',
        title: 'v0.5.0 — update alias for Decorator command from "d to de",',
        description: 'update alias for Decorator command from "d to de",',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.9',
        title: 'v0.4.9 — implement CI/CD pipeline, security auditing, and operational endpoints',
        description: 'implement CI/CD pipeline, security auditing, and operational endpoints',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'production-hardening-operational-endpoints',
        title: 'Production-hardening 2: Health probes, version endpoint, and build metadata',
        description: 'Liveness and readiness probe endpoints for Kubernetes (GET /health/live, GET /health/ready). Build metadata endpoint (GET /version) with git SHA, build timestamp, Rust version, and active features. HealthIndicator trait split for liveness vs readiness distinction.',
        date: 'Jul 16, 2026',
        tag: 'feature',
        readTime: '4 min',
    },
    {
        slug: 'production-hardening-ci-cd',
        title: 'Production-hardening 1: CI/CD pipeline, security auditing, and release automation',
        description: 'GitHub Actions CI with fmt/clippy/test/docs/audit/deny on every PR. Automated release workflow triggered by tag push. cargo-audit and cargo-deny for dependency vulnerability and license compliance. Local audit script for offline checks.',
        date: 'Jul 16, 2026',
        tag: 'release',
        readTime: '5 min',
    },
    {
        slug: 'v0.4.8',
        title: 'v0.4.8 — add database migration commands and update documentation',
        description: 'add database migration commands and update documentation',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.7',
        title: 'v0.4.7 — enhance release script and project generator for better version handling and documentation sync',
        description: 'enhance release script and project generator for better version handling and documentation sync',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.6',
        title: 'v0.4.6 — Release v0.4.6',
        description: 'Release v0.4.6',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.5',
        title: 'v0.4.5 — Release v0.4.5',
        description: 'Release v0.4.5',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.4',
        title: 'v0.4.4 — Release v0.4.4',
        description: 'Release v0.4.4',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.3',
        title: 'v0.4.3 — Release v0.4.3',
        description: 'Release v0.4.3',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.2',
        title: 'v0.4.2 — Release v0.4.2',
        description: 'Release v0.4.2',
        date: '2026-07-16',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.1',
        title: 'v0.4.1 — Release v0.4.1',
        description: 'Release v0.4.1',
        date: '2026-07-15',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.4.0',
        title: 'v0.4.0 — Production Readiness & Enterprise Features',
        description: 'Multipart uploads, Redis sessions, OAuth2 callback handler, backpressure, config hot-reload, error backtraces, and 15+ documentation pages',
        date: '2026-07-15',
        tag: 'major',
        readTime: '5 min',
    },
    {
        slug: 'compile-time-runtime',
        title: 'How Ironic resolves dependencies at runtime — no decorator magic needed',
        description: 'A deep dive into conditional providers, scoped instances, and how Ironic\u2019s compile-time wiring leaves runtime flexibility intact. Factories, OnceCell, and the blueprint-vs-construction metaphor.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'request-pipeline-internals',
        title: 'Inside the Ironic request pipeline — from HTTP to handler and back',
        description: 'Follow a single request through every stage: middleware onion, guard checkpoint, interceptor wrapping, parameter extraction, pipe chain, and handler dispatch. With real code from the source.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '12 min',
    },
    {
        slug: 'what-injectable-generates',
        title: 'What #[derive(Injectable)] actually generates — a line-by-line breakdown',
        description: 'See the exact Rust code the proc macro outputs. How struct fields become Dependency vecs, scope attributes become Scope enums, and Option<Arc<T>> becomes optional resolution.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '10 min',
    },
    {
        slug: 'zero-cost-feature-flags',
        title: 'Zero-cost feature flags — how Ironic eliminates dead code at compile time',
        description: 'How #[cfg(feature = "...")] prunes entire subsystems. Binary size comparison, cargo tree pruning, and why you cannot add features at runtime — and why you do not need to.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'module-graph-compilation',
        title: 'Module graph compilation — how Ironic validates your entire app before it starts',
        description: 'Topological sort, import resolution, provider visibility, circular dependency detection — all before the server binds to a port. See how compile_module_graph() works.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'type-erased-handler-dispatch',
        title: 'Type-erased yet type-safe — how Ironic dispatches handlers without reflection',
        description: 'How Box<dyn Any> extraction and downcast work together. Why type mismatch is impossible, how ParameterExtractor extracts path/query/body/header params, and where the nanoseconds go.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'platform-adapter-boundary',
        title: 'The platform adapter boundary — how Ironic talks to Axum',
        description: 'The HttpPlatformAdapter trait, how AxumAdapter converts compiled routes into Axum Router, the configure_router() escape hatch, and how you could swap Axum for anything else.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'lifecycle-orchestration',
        title: 'Lifecycle orchestration — how Ironic boots, runs, and shuts down deterministically',
        description: 'OnModuleInit cascade, eager provider initialization, partial failure rollback, Ctrl-C signal propagation, and reverse-order OnModuleDestroy. The exact order of every lifecycle callback.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'cli-code-generation-internals',
        title: 'How ironic generate patches your source code — AST-level surgery',
        description: 'Using syn to parse, insert, and rewrite Rust source files. How ensure_items and ensure_module_import perform idempotent, conflict-free code generation for modules, controllers, and resources.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'exception-filters-internals',
        title: 'Exception Filters — typed error handling with scope precedence',
        description: 'How ExceptionFilterSet chains route→controller→global filters, how FilterContext provides error-time route metadata, and how unhandled errors propagate through the middleware onion.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '9 min',
    },
    {
        slug: 'guard-composition',
        title: 'Guard Composition — how multiple guards form a single access decision',
        description: 'All-must-Allow semantics, short-circuit on first Deny, middleware unwind on guard denial, and the global-before-local index interleaving system.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'pipe-system-internals',
        title: 'The Pipe System — validation and transformation chains for erased values',
        description: 'How SyncPipe wraps closures into the ParameterPipe trait. Built-in parsers (int/float/bool/uuid), ValidationPipe with garde, and how pipes chain global→controller→route.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '9 min',
    },
    {
        slug: 'response-serialization-contract',
        title: 'Response and IntoResponse — the protocol-neutral boundary',
        description: 'How handler return values become responses, the structured error JSON format, and why the adapter never knows about Ironic error codes.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '6 min',
    },
    {
        slug: 'pipeline-components-scoping',
        title: 'PipelineComponents — how middleware, guards, and interceptors span three scopes',
        description: 'How PipelineComponents stores Vec<Arc<dyn Trait>> collections, how ExecutionState interleaves global and route-local components by index offset, and why all three scopes share the same type.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'security-middleware-internals',
        title: 'Security Middleware Internals — four independent defense layers',
        description: 'CORS deny-by-default origin allowlist, CSRF synchronizer token, InMemoryRateLimiter sliding window with per-client HashMap, and 9 secure default headers with per-header opt-out.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '10 min',
    },
    {
        slug: 'request-ids-tracing-spans',
        title: 'Request IDs and Tracing Spans — how observability bootstraps itself',
        description: 'The rf-{timestamp:032x}-{sequence:016x} ID format, atomic counter uniqueness without UUID overhead, tracing::info_span! propagation, and response correlation headers.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '6 min',
    },
    {
        slug: 'openapi-schema-generation',
        title: 'OpenAPI 3.1 Schema Generation — from compiled routes to Swagger UI',
        description: 'How OpenApiDocument walks compiled routes, reads OpenApiOperation metadata, generates JSON Schema from Rust types via the OpenApiSchema trait, and serves inline Swagger UI as a compile-time string.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '9 min',
    },
    {
        slug: 'field-level-json-redaction',
        title: 'Field-level JSON Redaction — how SerializeInterceptor guards sensitive data',
        description: 'How FieldRules with dotted-path traversal recursively apply Exclude and role-based Expose rules to serde_json::Value trees, including array element traversal.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'async-module-configuration',
        title: 'Asynchronous Module Configuration — deferred root modules from remote sources',
        description: 'How module_async() accepts a future, how RootModule::Deferred awaits secret manager responses before compile_module_graph() runs, and how ModuleConfigurationError sanitizes credential leaks.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '6 min',
    },
    {
        slug: 'in-process-testing-internals',
        title: 'In-Process Testing Without a Socket — how TestApplication works',
        description: 'How InProcessAdapter skips TCP, how match_path() does segment-by-segment :param extraction, how TestResponse assertions work, and how Drop runs cleanup on panic.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '9 min',
    },
    {
        slug: 'websocket-connection-lifecycle',
        title: 'WebSocket Connection Lifecycle — clients, rooms, and broadcasting',
        description: 'How WsConnections uses Arc<RwLock<HashMap<ClientId, mpsc::UnboundedSender>>>, atomic ClientId counters, dual-lock room broadcasting, and disconnect room scanning.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'native-websocket-sse-helpers',
        title: 'Native WebSocket and SSE Helpers — adapter-level realtime escapes',
        description: 'The WebSocketHandler trait, Axum on_upgrade dance, and how sse_channel() bridges an mpsc channel into an Axum SSE response body.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '5 min',
    },
    {
        slug: 'layered-configuration-system',
        title: 'Layered Configuration — TOML, JSON, env vars, and secret redaction',
        description: 'How ConfigurationLoader merges TOML/JSON/env in priority order with __ nested separators, how ValidateConfiguration adds custom rules, and how Secret<T> redacts everywhere except explicit expose.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'provider-override-system',
        title: 'Provider Override System — how test mocks and production config swaps work',
        description: 'How ProviderKey matching, three override strategies (provider/value/factory), and the shared Vec<ProviderDefinition> pipeline work identically across TestApplicationBuilder, ApplicationBuilder, and TestModuleBuilder.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '6 min',
    },
    {
        slug: 'ironic-main-macro',
        title: '#[ironic::main] — Tokio runtime bootstrapping via macro surgery',
        description: 'How the proc macro strips async and wraps the body in block_on, creating a multi-thread Tokio runtime so users write async fn main() without ever importing Tokio.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'module-ref-lazy-injection',
        title: 'ModuleRef — lazy container injection via Arc<OnceLock<Container>>',
        description: 'Solves the DI chicken-and-egg problem. ModuleRef is registered as a provider before the container exists, then populated after build. Services call resolve() lazily at runtime.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'scope-violation-captive-prevention',
        title: 'ScopeViolation — how the resolver prevents singletons from capturing request-scoped state',
        description: 'The request_allowed flag propagates false through singleton boundaries. Any attempt to resolve a request-scoped provider from a singleton factory returns ScopeViolation — runtime safety without lifetimes.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'oncecell-singleton-cancel-safe',
        title: 'OnceCell-based singletons — cancellation-safe initialization and retry',
        description: 'How get_or_try_init enables both failed-init retry and cancelled-init retry. Why task.abort() and select! dropping futures does not poison the singleton.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'derive-module-custom-dsl',
        title: '#[derive(Module)] — parsing a custom DSL in proc macro attributes',
        description: 'How syn::Parse tokenizes #[module(imports = [...], providers = [...], controllers = [...], exports = [...]]) plus the separate #[global] attribute into chained builder method calls.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'two-phase-route-compilation',
        title: 'Two-phase route compilation — path normalization and cross-controller conflict detection',
        description: 'Phase 1 normalizes paths and checks intra-controller duplicates. Phase 2 checks cross-controller conflicts with HashSet<(HttpMethod, String, Option<VersionMetadata>)> — versioned routes share paths.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'cqrs-typed-dispatch',
        title: 'CQRS Bus — TypeId-based typed command and query dispatch',
        description: 'How HashMap<TypeId, Arc<dyn Fn>> stores handlers, how downcast::<Input>() and downcast::<Output>() recover concrete types, and how one-handler-per-message-type is enforced.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'event-bus-phantom-gc',
        title: 'Typed Event Bus — phantom subscription filtering and channel garbage collection',
        description: 'How PhantomData<fn() -> E> provides compile-time dispatch, how recv() silently discards non-matching events via downcast, and how publish() garbage-collects closed subscriber channels.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '7 min',
    },
    {
        slug: 'sagas-reverse-compensation',
        title: 'Sagas — ordered steps with reverse compensation and error priority',
        description: 'How steps run sequentially, on failure previously completed steps compensate in REVERSE order, and how compensation errors take priority over execution errors — manual recovery required.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'static-plugin-system',
        title: 'Static Plugin System — linked module composition with name collision prevention',
        description: 'How PluginRegistry deduplicates by HashSet<&str>, how apply_all() uses linear fold over ModuleDefinitionBuilder, and how each Plugin contributes providers/controllers/imports to any app.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
    },
    {
        slug: 'lifecycle-axum-wiring',
        title: 'Lifecycle hooks under the hood — how OnModuleInit connects to axum::serve',
        description: 'A full trace of how lifecycle hooks run OUTSIDE Axum\u2019s stack. Init before the listener binds. Shutdown after serve returns. The initialized Vec as the bridge between build and destroy.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '10 min',
    },
    {
        slug: 'v0.3.9',
        title: 'Release automation — blog posts, comparison table, and GitHub badges',
        description: 'Every release now auto-generates a blog post and updates the releases pages. Added framework comparison table and live GitHub star/fork counts.',
        date: 'Jul 15, 2026',
        tag: 'release',
        readTime: '2 min',
    },
    {
        slug: 'v0.3.8',
        title: 'Batteries included — security, middleware, and Docker out of the box',
        description: 'Every new project now ships with SecurityHeaders, RateLimit, and CORS middleware. The Dockerfile is production-ready with proper binary naming and SERVER_HOST=0.0.0.0.',
        date: 'Jul 15, 2026',
        tag: 'release',
        readTime: '3 min',
    },
    {
        slug: 'v0.3.7',
        title: 'Global middleware arrives on ApplicationBuilder',
        description: 'The .middleware() builder method lets you register any impl Middleware from main.rs. No more per-controller registration for app-wide concerns.',
        date: 'Jul 15, 2026',
        tag: 'feature',
        readTime: '2 min',
    },
    {
        slug: 'v0.3.6',
        title: 'Validation pipes, API examples, and expanded project scaffolding',
        description: 'Comprehensive garde validation docs, two new example apps (basic CRUD and full auth), and serde/garde/dotenvy dependencies included in every new project.',
        date: 'Jul 15, 2026',
        tag: 'release',
        readTime: '3 min',
    },
    {
        slug: 'v0.3.5',
        title: 'Auth test restructuring for cleaner module organization',
        description: 'Authentication test files reorganized with proper module imports for better maintainability.',
        date: 'Jul 15, 2026',
        tag: 'fix',
        readTime: '1 min',
    },
    {
        slug: 'v0.3.4',
        title: 'Documentation site goes live with SPA fallback',
        description: 'The docs site is deployed to GitHub Pages with .nojekyll and proper client-side routing support via 404.html fallback.',
        date: 'Jul 15, 2026',
        tag: 'feature',
        readTime: '2 min',
    },
    {
        slug: 'v0.3.3',
        title: 'Smart code generation — auto-add dependencies on scaffold',
        description: 'ironic generate now automatically adds required crates to Cargo.toml when scaffolding new modules. No more manual dependency edits.',
        date: 'Jul 15, 2026',
        tag: 'feature',
        readTime: '2 min',
    },
    {
        slug: 'v0.3.0',
        title: 'First stable release — the complete Rust application framework',
        description: 'Module graph compiler, DI container, controller routing, request pipeline, CLI generator, socket-free testing, and lifecycle hooks — all built on Axum with zero-cost compile-time feature flags.',
        date: 'Jul 15, 2026',
        tag: 'major',
        readTime: '5 min',
    },
    {
        slug: 'v0.1.x-v0.2.x',
        title: 'The pre-release journey — from 9-crate workspace to full framework',
        description: 'How Ironic evolved from the first public release through 19 iterations: authentication modules, file upload, email, ready-resource generators, and NestJS feature parity.',
        date: 'Jul 14, 2026',
        tag: 'major',
        readTime: '4 min',
    },
];

const tagStyles: Record<Post['tag'], { icon: React.ReactNode; className: string }> = {
    major: {
        icon: <Sparkles className="size-3" />,
        className: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300 border-amber-200 dark:border-amber-800',
    },
    release: {
        icon: <Layers className="size-3" />,
        className: 'bg-brand/10 text-brand border-brand/20',
    },
    feature: {
        icon: <Star className="size-3" />,
        className: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800',
    },
    fix: {
        icon: <GitBranch className="size-3" />,
        className: 'bg-sky-100 text-sky-800 dark:bg-sky-900/30 dark:text-sky-300 border-sky-200 dark:border-sky-800',
    },
    'deep-dive': {
        icon: <Sparkles className="size-3" />,
        className: 'bg-violet-100 text-violet-800 dark:bg-violet-900/30 dark:text-violet-300 border-violet-200 dark:border-violet-800',
    },
};

export default function BlogIndex() {
    const featured = posts[0];
    const rest = posts.slice(1);

    return (
        <div className="bg-fd-background">
            {/* Hero */}
            <section className="relative overflow-hidden border-b border-fd-border">
                <div className="absolute inset-0 pointer-events-none">
                    <div className="absolute top-0 right-0 w-125 h-125 bg-brand/3 rounded-full blur-[100px] translate-x-1/4 -translate-y-1/4" />
                    <div className="absolute bottom-0 left-0 w-100 h-100 bg-amber-500/3 rounded-full blur-[80px] -translate-x-1/4 translate-y-1/4" />
                </div>
                <div className="max-w-7xl mx-auto px-6 py-20 md:py-28 relative">
                    <div className="max-w-2xl">
                        <div className="inline-flex items-center gap-2 rounded-full border border-fd-border bg-fd-card/60 px-4 py-1.5 text-xs font-medium text-fd-muted-foreground mb-6">
                            <span className="relative flex h-2 w-2">
                                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75" />
                                <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500" />
                            </span>
                            {LATEST_VERSION_LABEL}
                        </div>
                        <h1 className="text-4xl md:text-5xl lg:text-6xl font-extrabold text-fd-foreground tracking-tight leading-[1.1] mb-6">
                            The Ironic
                            <br />
                            <span className="text-brand">Release Log</span>
                        </h1>
                        <p className="text-base md:text-lg text-fd-muted-foreground leading-relaxed max-w-xl">
                            Every version ships with production-ready defaults. Follow along for new features, security improvements, and developer experience upgrades.
                        </p>
                    </div>
                </div>
            </section>

            {/* Posts */}
            <div className="max-w-7xl mx-auto px-6 py-16">
                {/* Featured post */}
                <div className="mb-16">
                    <p className="text-xs font-semibold text-fd-muted-foreground uppercase tracking-widest mb-4">Latest release</p>
                    <Link
                        to={`/blog/${featured.slug}`}
                        className="group block relative overflow-hidden rounded-2xl border border-fd-border bg-fd-card/50 hover:border-brand/30 transition-all duration-300"
                    >
                        <div className="absolute inset-0 bg-linear-to-br from-brand/4 via-transparent to-amber-500/3 group-hover:opacity-100 transition-opacity" />
                        <div className="relative p-8 md:p-10 md:flex md:gap-10 md:items-center">
                            <div className="flex-1">
                                <div className="flex items-center gap-3 mb-4">
                                    <span className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[11px] font-semibold ${tagStyles[featured.tag].className}`}>
                                        {tagStyles[featured.tag].icon}
                                        {featured.tag}
                                    </span>
                                    <span className="inline-flex items-center gap-1.5 text-[11px] text-fd-muted-foreground">
                                        <Calendar className="size-3" />
                                        {featured.date}
                                    </span>
                                    <span className="inline-flex items-center gap-1.5 text-[11px] text-fd-muted-foreground">
                                        <Clock className="size-3" />
                                        {featured.readTime}
                                    </span>
                                </div>
                                <h2 className="text-2xl md:text-3xl font-bold text-fd-foreground tracking-tight mb-3 group-hover:text-brand transition-colors">
                                    {featured.title}
                                </h2>
                                <p className="text-fd-muted-foreground leading-relaxed">
                                    {featured.description}
                                </p>
                            </div>
                            <div className="hidden md:flex shrink-0 items-center justify-center w-14 h-14 rounded-full border border-brand/20 bg-brand/10 text-brand group-hover:bg-brand group-hover:text-white transition-all duration-300 group-hover:scale-110">
                                <ArrowRight className="size-5" />
                            </div>
                        </div>
                    </Link>
                </div>

                {/* Post grid */}
                <p className="text-xs font-semibold text-fd-muted-foreground uppercase tracking-widest mb-6">Previous releases</p>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                    {rest.map((post) => (
                        <Link
                            key={post.slug}
                            to={`/blog/${post.slug}`}
                            className="group flex flex-col rounded-xl border border-fd-border bg-fd-card/30 hover:border-brand/30 hover:bg-fd-card/60 transition-all duration-300"
                        >
                            <div className="p-6 flex flex-col flex-1">
                                <div className="flex items-center gap-3 mb-3">
                                    <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tagStyles[post.tag].className}`}>
                                        {tagStyles[post.tag].icon}
                                        {post.tag}
                                    </span>
                                </div>
                                <h3 className="text-base font-semibold text-fd-foreground tracking-tight mb-2 group-hover:text-brand transition-colors line-clamp-2">
                                    {post.title}
                                </h3>
                                <p className="text-sm text-fd-muted-foreground leading-relaxed mb-4 line-clamp-3 flex-1">
                                    {post.description}
                                </p>
                                <div className="flex items-center justify-between pt-4 border-t border-fd-border/50">
                                    <div className="flex items-center gap-3 text-[11px] text-fd-muted-foreground">
                                        <span className="inline-flex items-center gap-1">
                                            <Calendar className="size-3" />
                                            {post.date}
                                        </span>
                                        <span className="inline-flex items-center gap-1">
                                            <Clock className="size-3" />
                                            {post.readTime}
                                        </span>
                                    </div>
                                    <ArrowRight className="size-4 text-fd-muted-foreground group-hover:text-brand group-hover:translate-x-0.5 transition-all" />
                                </div>
                            </div>
                        </Link>
                    ))}
                </div>
            </div>

            {/* Subscribe CTA */}
            <section className="border-t border-fd-border">
                <div className="max-w-7xl mx-auto px-6 py-16">
                    <div className="relative rounded-2xl border border-fd-border bg-fd-card/40 p-10 md:p-14 text-center overflow-hidden">
                        <div className="absolute inset-0 bg-linear-to-br from-brand/5 via-transparent to-amber-500/3 pointer-events-none" />
                        <div className="relative max-w-md mx-auto">
                            <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-brand/10 text-brand mb-5">
                                <Sparkles className="size-6" />
                            </div>
                            <h2 className="text-2xl font-bold text-fd-foreground tracking-tight mb-3">
                                Stay up to date
                            </h2>
                            <p className="text-sm text-fd-muted-foreground leading-relaxed mb-6">
                                Star the repo on GitHub to get notified of new releases, or watch the repository for updates.
                            </p>
                            <a
                                href="https://github.com/ironic-org/ironic"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="inline-flex items-center gap-2 rounded-full bg-fd-foreground text-fd-background px-6 py-2.5 text-sm font-semibold hover:bg-fd-foreground/90 transition-colors"
                            >
                                <Star className="size-4 fill-current" />
                                Star on GitHub
                            </a>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    );
}
