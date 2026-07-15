import { ArrowRight, Calendar, Tag } from 'lucide-react';

const posts = [
    {
        slug: 'v0.3.8',
        title: 'v0.3.8',
        description: 'Global middleware, security headers, rate limiting, CORS, and Docker fixes — all enabled by default in new projects.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.7',
        title: 'v0.3.7',
        description: 'FrameworkApplicationBuilder now supports .middleware() for registering global middleware from main.rs.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.6',
        title: 'v0.3.6',
        description: 'Validation pipes documentation, new auth/basic CRUD example apps, and expanded project scaffolding.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.5',
        title: 'v0.3.5',
        description: 'Authentication test file restructuring for better module organization.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.4',
        title: 'v0.3.4',
        description: 'Documentation site deployed with SPA fallback, integration test paths fixed.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.3',
        title: 'v0.3.3',
        description: 'CLI generator auto-adds required Cargo dependencies when scaffolding new modules.',
        date: '2026-07-15',
    },
    {
        slug: 'v0.3.0',
        title: 'v0.3.0 - v0.3.2',
        description: 'First stable release — module system, DI container, controller routing, CLI generator, testing harness, and lifecycle hooks.',
        date: '2026-07-15',
    },
];

export default function BlogIndex() {
    return (
        <div className="max-w-3xl mx-auto px-6 py-16">
            <div className="mb-12">
                <h1 className="text-4xl font-bold text-fd-foreground tracking-tight mb-3">
                    Release Notes
                </h1>
                <p className="text-fd-muted-foreground text-base">
                    Version changes, new features, and updates for the Ironic framework.
                </p>
            </div>

            <div className="space-y-6">
                {posts.map((post) => (
                    <a
                        key={post.slug}
                        href={`/blog/${post.slug}`}
                        className="block group rounded-xl border border-fd-border bg-fd-card/40 p-6 hover:border-brand/30 hover:bg-fd-card/80 transition-all"
                    >
                        <div className="flex items-center gap-3 mb-2">
                            <span className="inline-flex items-center gap-1.5 rounded-full bg-brand/10 border border-brand/20 px-3 py-0.5 text-xs font-semibold text-brand">
                                <Tag className="size-3" />
                                {post.title}
                            </span>
                            <span className="inline-flex items-center gap-1 text-xs text-fd-muted-foreground">
                                <Calendar className="size-3" />
                                {post.date}
                            </span>
                        </div>
                        <p className="text-sm text-fd-muted-foreground leading-relaxed mb-3">
                            {post.description}
                        </p>
                        <span className="inline-flex items-center gap-1 text-xs font-medium text-brand group-hover:gap-2 transition-all">
                            Read more
                            <ArrowRight className="size-3" />
                        </span>
                    </a>
                ))}
            </div>
        </div>
    );
}
