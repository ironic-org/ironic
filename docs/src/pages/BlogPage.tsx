import { useMemo } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { getMDXComponents } from '@/mdx-components';
import { blogSource } from '@/lib/source';
import { ArrowLeft, Calendar, Clock, Github } from 'lucide-react';
import BlogIndex from './BlogIndex';

function getBlogSlug(pathname: string): string[] | undefined {
    const stripped = pathname.replace(/^\/blog\/?/, '').replace(/\/$/, '');
    if (!stripped) return undefined;
    return stripped.split('/').filter(Boolean);
}

export default function BlogPage() {
    const location = useLocation();
    const slug = useMemo(() => getBlogSlug(location.pathname), [location.pathname]);
    const page = slug ? blogSource.getPage(slug) : undefined;
    const pageData = page?.data;

    if (!slug) {
        return (
            <div className="min-h-screen bg-fd-background">
                <BlogIndex />
            </div>
        );
    }

    if (!page || !pageData) {
        return (
            <div className="min-h-screen bg-fd-background flex items-center justify-center">
                <div className="text-center px-6 py-16 max-w-md">
                    <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-fd-muted/30 text-fd-muted-foreground mb-6">
                        <span className="text-3xl font-bold">404</span>
                    </div>
                    <h1 className="text-2xl font-bold text-fd-foreground mb-2">Post not found</h1>
                    <p className="text-fd-muted-foreground text-sm leading-relaxed mb-6">
                        This post may have been moved or renamed. Check the blog index for all available releases.
                    </p>
                    <Link
                        to="/blog"
                        className="inline-flex items-center gap-2 rounded-full bg-brand px-5 py-2 text-sm font-semibold text-white hover:bg-brand/90 transition-colors"
                    >
                        <ArrowLeft className="size-4" />
                        Browse all posts
                    </Link>
                </div>
            </div>
        );
    }

    const Body = pageData.body;
    if (!Body) return null;

    return (
        <div className="min-h-screen bg-fd-background">
            {/* Article header */}
            <div className="relative overflow-hidden border-b border-fd-border">
                <div className="absolute inset-0 pointer-events-none">
                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-brand/[0.03] rounded-full blur-[100px]" />
                </div>
                <div className="max-w-3xl mx-auto px-6 py-16 md:py-20 relative">
                    <Link
                        to="/blog"
                        className="inline-flex items-center gap-1.5 text-sm font-medium text-fd-muted-foreground hover:text-brand mb-8 transition-colors group"
                    >
                        <ArrowLeft className="size-4 group-hover:-translate-x-0.5 transition-transform" />
                        Back to blog
                    </Link>

                    <h1 className="text-3xl md:text-4xl font-extrabold text-fd-foreground tracking-tight leading-[1.15] mb-4">
                        {pageData.title}
                    </h1>
                    {pageData.description && (
                        <p className="text-base md:text-lg text-fd-muted-foreground leading-relaxed max-w-2xl">
                            {pageData.description}
                        </p>
                    )}

                    <div className="flex flex-wrap items-center gap-4 mt-6 pt-6 border-t border-fd-border/50">
                        <div className="flex items-center gap-2 text-xs text-fd-muted-foreground">
                            <Calendar className="size-3.5" />
                            Jul 15, 2026
                        </div>
                        <div className="flex items-center gap-2 text-xs text-fd-muted-foreground">
                            <Clock className="size-3.5" />
                            3 min read
                        </div>
                        <a
                            href="https://github.com/ironic-org/ironic"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1.5 text-xs font-medium text-fd-muted-foreground hover:text-fd-foreground transition-colors ml-auto"
                        >
                            <Github className="size-3.5" />
                            View on GitHub
                        </a>
                    </div>
                </div>
            </div>

            {/* Article body */}
            <div className="max-w-3xl mx-auto px-6 py-16">
                <article className="prose prose-neutral dark:prose-invert max-w-none
                    prose-headings:font-bold prose-headings:tracking-tight
                    prose-h2:text-2xl prose-h2:mt-12 prose-h2:mb-4
                    prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3
                    prose-p:leading-relaxed prose-p:my-4
                    prose-code:before:content-none prose-code:after:content-none
                    prose-code:bg-fd-muted/50 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded
                    prose-pre:rounded-xl prose-pre:border prose-pre:border-fd-border
                    prose-table:rounded-xl prose-table:overflow-hidden
                    prose-th:bg-fd-muted/30 prose-th:px-4 prose-th:py-2 prose-th:text-xs prose-th:font-semibold
                    prose-td:px-4 prose-td:py-2 prose-td:text-sm
                    prose-strong:text-fd-foreground
                    prose-blockquote:border-l-brand prose-blockquote:bg-fd-muted/20 prose-blockquote:rounded-r-lg prose-blockquote:py-3 prose-blockquote:px-4
                    prose-li:my-1
                ">
                    <Body components={getMDXComponents()} />
                </article>

                {/* Bottom nav */}
                <div className="mt-16 pt-8 border-t border-fd-border flex items-center justify-between">
                    <Link
                        to="/blog"
                        className="inline-flex items-center gap-2 text-sm font-medium text-fd-muted-foreground hover:text-brand transition-colors group"
                    >
                        <ArrowLeft className="size-4 group-hover:-translate-x-0.5 transition-transform" />
                        All releases
                    </Link>
                    <a
                        href="https://github.com/ironic-org/ironic/discussions"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-2 text-sm font-medium text-fd-muted-foreground hover:text-brand transition-colors"
                    >
                        Discuss this release
                        <Github className="size-4" />
                    </a>
                </div>
            </div>
        </div>
    );
}
