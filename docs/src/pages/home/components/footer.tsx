import { Github, Layers } from 'lucide-react';
import { Link } from 'react-router-dom';

const footerLinks = {
    Docs: [
        { label: 'Getting started', href: '/docs/getting-started/getting-started' },
        { label: 'Fundamentals', href: '/docs/core/fundamentals' },
        { label: 'CLI', href: '/docs/getting-started/cli' },
        { label: 'API Reference', href: '/docs' },
    ],
    Features: [
        { label: 'Security', href: '/docs/http-api/security' },
        { label: 'Cache', href: '/docs/performance/cache-decorators' },
        { label: 'Scheduling', href: '/docs/performance/scheduling' },
        { label: 'WebSockets', href: '/docs/advanced/websocket-gateways' },
    ],
    Resources: [
        { label: 'GitHub', href: 'https://github.com/ironic-org/ironic' },
        { label: 'Benchmarks', href: '/docs/more/benchmarks' },
        { label: 'Examples', href: '/docs/more/examples' },
        { label: 'CHANGELOG', href: 'https://github.com/ironic-org/ironic/blob/main/CHANGELOG.md' },
    ],
};

const Footer = () => {
    return (
        <footer className='relative z-10 border-t border-fd-border bg-fd-background'>
            <div className='mx-auto max-w-7xl px-6 py-16'>
                <div className='grid grid-cols-1 gap-10 sm:grid-cols-2 lg:grid-cols-4'>
                    <div>
                        <div className='flex items-center gap-2 mb-4'>
                            <span className='flex items-center justify-center rounded-lg border border-brand/20 bg-brand/10 p-1.5 text-brand'>
                                <Layers className='h-5 w-5' />
                            </span>
                            <span className='text-lg font-bold tracking-tight text-fd-foreground'>
                                Ironic
                            </span>
                        </div>
                        <p className='text-sm text-fd-muted-foreground leading-relaxed mb-4'>
                            A type-safe application framework for Rust.
                            Built on Axum, designed for production.
                        </p>
                        <a
                            href='https://github.com/ironic-org/ironic'
                            target='_blank'
                            rel='noopener noreferrer'
                            className='inline-flex items-center gap-2 text-sm text-fd-muted-foreground hover:text-fd-foreground transition-colors'>
                            <Github className='w-4 h-4' />
                            ironic-org/ironic
                        </a>
                    </div>

                    {Object.entries(footerLinks).map(([category, links]) => (
                        <div key={category}>
                            <h4 className='text-xs font-bold text-fd-foreground uppercase tracking-wider mb-4'>
                                {category}
                            </h4>
                            <ul className='space-y-2'>
                                {links.map((link) => (
                                    <li key={link.label}>
                                        {link.href.startsWith('/') ? (
                                            <Link
                                                to={link.href}
                                                className='text-sm text-fd-muted-foreground hover:text-fd-foreground transition-colors'>
                                                {link.label}
                                            </Link>
                                        ) : (
                                            <a
                                                href={link.href}
                                                target='_blank'
                                                rel='noopener noreferrer'
                                                className='text-sm text-fd-muted-foreground hover:text-fd-foreground transition-colors'>
                                                {link.label}
                                            </a>
                                        )}
                                    </li>
                                ))}
                            </ul>
                        </div>
                    ))}
                </div>

                <div className='mt-12 pt-8 border-t border-fd-border flex flex-col sm:flex-row items-center justify-between gap-4'>
                    <p className='text-xs text-fd-muted-foreground'>
                        Released under the MIT License. Copyright &copy; {new Date().getFullYear()} Ironic contributors.
                    </p>
                    <p className='text-xs text-fd-muted-foreground'>
                        Built with Rust + Axum
                    </p>
                </div>
            </div>
        </footer>
    );
};

export default Footer;
