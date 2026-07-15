import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { Github, Star, GitFork } from 'lucide-react';
import { useEffect, useState } from 'react';

function formatCount(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
    return n.toString();
}

function GitHubStarsBadge() {
    const [stars, setStars] = useState<number | null>(null);
    const [forks, setForks] = useState<number | null>(null);

    useEffect(() => {
        fetch('https://api.github.com/repos/ironic-org/ironic')
            .then((res) => res.json())
            .then((data) => {
                setStars(data.stargazers_count ?? null);
                setForks(data.forks_count ?? null);
            })
            .catch(() => {});
    }, []);

    return (
        <a
            href='https://github.com/ironic-org/ironic'
            target='_blank'
            rel='noopener noreferrer'
            className='inline-flex items-center gap-1.5 text-fd-muted-foreground hover:text-fd-foreground transition-colors'
        >
            <Github className='size-4' />
            <span className='hidden xl:inline-flex items-center gap-0.5 rounded-full border border-fd-border bg-fd-card/50 px-0.5 py-0.5 text-[11px] font-medium'>
                <span className='inline-flex items-center gap-1 rounded-full bg-amber-400/10 px-1.5 py-0.5'>
                    <Star className='size-3 fill-amber-400 stroke-amber-500' />
                    <span className='text-amber-900 dark:text-amber-200 tabular-nums'>
                        {stars !== null ? formatCount(stars) : '—'}
                    </span>
                </span>
                <span className='inline-flex items-center gap-1 rounded-full bg-sky-400/10 px-1.5 py-0.5'>
                    <GitFork className='size-3 stroke-sky-500' />
                    <span className='text-sky-700 dark:text-sky-300 tabular-nums'>
                        {forks !== null ? formatCount(forks) : '—'}
                    </span>
                </span>
            </span>
        </a>
    );
}

export function baseOptions(): BaseLayoutProps {
    return {
        nav: {
            title: (
                <>
                    <svg
                        width='24'
                        height='24'
                        viewBox='0 0 24 24'
                        fill='none'
                        stroke='currentColor'
                        strokeWidth='2'
                        strokeLinecap='round'
                        strokeLinejoin='round'
                        className='size-5 text-brand'
                    >
                        <path d='M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5' />
                    </svg>
                    <span className='font-medium max-md:hidden'>Ironic</span>
                </>
            ),
            children: <GitHubStarsBadge />,
        },
    };
}
