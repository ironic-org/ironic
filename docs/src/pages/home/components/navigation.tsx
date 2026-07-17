import { useState } from 'react';
import { Github, Menu, X, GitBranch } from 'lucide-react';
import { Link } from 'react-router-dom';
import { GitHubStatsBadge } from './github-stars';
import { GIT_BRANCH } from '@/lib/constants';

const navLinks = [
    { label: 'Docs', href: '/docs/getting-started/getting-started', router: true },
    { label: 'Blog', href: '/blog', router: true },
    { label: 'Examples', href: '/docs/more/examples', router: true },
    { label: 'Releases', href: '/docs/releases', router: true },
];

function GitHubBadge() {
    return (
        <a
            href='https://github.com/ironic-org/ironic'
            target='_blank'
            rel='noopener noreferrer'
            className='inline-flex items-center gap-1.5 text-fd-muted-foreground hover:text-fd-foreground transition-colors group'
        >
            <Github className='h-5 w-5 group-hover:scale-110 transition-transform' />
            <GitHubStatsBadge />
        </a>
    );
}

const Navigation = () => {
    const [open, setOpen] = useState(false);

    return (
        <nav className='sticky top-0 z-50 w-full border-b border-fd-border bg-fd-background/80 backdrop-blur-md'>
            <div className='mx-auto flex h-16 max-w-7xl items-center justify-between px-6'>
                <Link to='/' className='flex items-center gap-2'>
                    <span className='flex items-center justify-center rounded-lg border border-brand/20 bg-brand/10 p-1.5'>
                        <img src='/logo.png' alt='Ironic' width='20' height='20' className='size-5' />
                    </span>
                    <span className='text-lg font-bold tracking-tight text-fd-foreground'>
                        Ironic
                    </span>
                    <span className='inline-flex items-center gap-1 rounded-full border border-emerald-200 dark:border-emerald-800 bg-emerald-50 dark:bg-emerald-950/50 px-2 py-0.5 text-[10px] font-medium text-emerald-700 dark:text-emerald-300 max-md:hidden'>
                        <span className='relative flex size-1.5'>
                            <span className='absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-75' />
                            <span className='relative inline-flex size-1.5 rounded-full bg-emerald-500' />
                        </span>
                        {GIT_BRANCH}
                    </span>
                </Link>

                <div className='hidden items-center gap-8 md:flex'>
                    {navLinks.map((link) =>
                        link.router ? (
                            <Link
                                key={link.label}
                                to={link.href}
                                className='text-sm font-medium text-fd-muted-foreground transition-colors hover:text-brand'>
                                {link.label}
                            </Link>
                        ) : (
                            <a
                                key={link.label}
                                href={link.href}
                                className='text-sm font-medium text-fd-muted-foreground transition-colors hover:text-brand'>
                                {link.label}
                            </a>
                        ),
                    )}
                    <GitHubBadge />
                </div>

                <button
                    onClick={() => setOpen(!open)}
                    className='md:hidden p-2 text-fd-muted-foreground hover:text-fd-foreground'
                    aria-label='Toggle menu'>
                    {open ? <X className='h-5 w-5' /> : <Menu className='h-5 w-5' />}
                </button>
            </div>

            {open && (
                <div className='md:hidden border-t border-fd-border bg-fd-background px-6 py-4 flex flex-col gap-3'>
                    {navLinks.map((link) =>
                        link.router ? (
                            <Link
                                key={link.label}
                                to={link.href}
                                onClick={() => setOpen(false)}
                                className='text-sm font-medium text-fd-muted-foreground hover:text-brand py-1'>
                                {link.label}
                            </Link>
                        ) : (
                            <a
                                key={link.label}
                                href={link.href}
                                onClick={() => setOpen(false)}
                                className='text-sm font-medium text-fd-muted-foreground hover:text-brand py-1'>
                                {link.label}
                            </a>
                        ),
                    )}
                    <div className='pt-2 border-t border-fd-border/50'>
                        <GitHubBadge />
                    </div>
                </div>
            )}
        </nav>
    );
};

export default Navigation;
