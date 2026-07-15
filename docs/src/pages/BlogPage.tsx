import { useMemo } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { getMDXComponents } from '@/mdx-components';
import { blogSource } from '@/lib/source';
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
    const body = pageData?.body ? <pageData.body components={getMDXComponents()} /> : null;

    if (!slug) {
        return (
            <div className="min-h-screen bg-fd-background">
                <div className="max-w-7xl mx-auto">
                    <BlogIndex />
                </div>
            </div>
        );
    }

    if (!page) {
        return (
            <div className="max-w-3xl mx-auto px-6 py-16">
                <h1 className="text-3xl font-bold text-fd-foreground">Post not found</h1>
                <p className="mt-2 text-fd-muted-foreground">
                    The blog post you're looking for doesn't exist.
                </p>
                <Link to="/blog" className="inline-block mt-6 text-sm font-medium text-brand hover:underline">
                    ← Back to blog
                </Link>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-fd-background">
            <div className="max-w-3xl mx-auto px-6 py-16">
                <Link to="/blog" className="inline-flex items-center gap-1 text-sm font-medium text-fd-muted-foreground hover:text-brand mb-8 transition-colors">
                    ← Back to blog
                </Link>
                <article className="prose prose-neutral dark:prose-invert max-w-none">
                    {body}
                </article>
            </div>
        </div>
    );
}
