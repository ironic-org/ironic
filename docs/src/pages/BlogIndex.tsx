import { ArrowRight, Calendar, Clock, GitBranch, Layers, Sparkles, Star } from 'lucide-react';
import { Link } from 'react-router-dom';

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
        slug: 'compile-time-runtime',
        title: 'How Ironic resolves dependencies at runtime — no decorator magic needed',
        description: 'A deep dive into conditional providers, scoped instances, and how Ironic\u2019s compile-time wiring leaves runtime flexibility intact. Factories, OnceCell, and the blueprint-vs-construction metaphor.',
        date: 'Jul 15, 2026',
        tag: 'deep-dive',
        readTime: '8 min',
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
        title: 'Global middleware arrives on FrameworkApplicationBuilder',
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
                    <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-brand/[0.03] rounded-full blur-[100px] translate-x-1/4 -translate-y-1/4" />
                    <div className="absolute bottom-0 left-0 w-[400px] h-[400px] bg-amber-500/[0.03] rounded-full blur-[80px] -translate-x-1/4 translate-y-1/4" />
                </div>
                <div className="max-w-7xl mx-auto px-6 py-20 md:py-28 relative">
                    <div className="max-w-2xl">
                        <div className="inline-flex items-center gap-2 rounded-full border border-fd-border bg-fd-card/60 px-4 py-1.5 text-xs font-medium text-fd-muted-foreground mb-6">
                            <span className="relative flex h-2 w-2">
                                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75" />
                                <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500" />
                            </span>
                            Latest: v0.3.8
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
                        <div className="absolute inset-0 bg-gradient-to-br from-brand/[0.04] via-transparent to-amber-500/[0.03] group-hover:opacity-100 transition-opacity" />
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
                        <div className="absolute inset-0 bg-gradient-to-br from-brand/[0.05] via-transparent to-amber-500/[0.03] pointer-events-none" />
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
