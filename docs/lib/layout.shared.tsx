import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { Github, Star, GitFork, GitBranch } from 'lucide-react';
import { useEffect, useState } from 'react';
import { GIT_BRANCH } from './constants';

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
                    <img
                        src='/logo.png'
                        alt='Ironic'
                        width='24'
                        height='24'
                        className='size-5'
                    />
                    <span className='font-medium max-md:hidden'>Ironic</span>
                    <span className='inline-flex items-center gap-1 rounded-full border border-emerald-200 dark:border-emerald-800 bg-emerald-50 dark:bg-emerald-950/50 px-2 py-0.5 text-[10px] font-medium text-emerald-700 dark:text-emerald-300 ml-1.5 max-md:hidden'>
                        <span className='relative flex size-1.5'>
                            <span className='absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-75' />
                            <span className='relative inline-flex size-1.5 rounded-full bg-emerald-500' />
                        </span>
                        {GIT_BRANCH}
                    </span>
                </>
            ),
            children: <GitHubStarsBadge />,
        },
    };
}
