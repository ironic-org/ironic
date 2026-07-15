import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { Github, Star, GitFork } from 'lucide-react';
import { useEffect, useState } from 'react';

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
            className='inline-flex items-center gap-2 text-fd-muted-foreground hover:text-fd-foreground transition-colors text-sm'
        >
            <Github className='size-4' />
            <span className='hidden xl:inline-flex items-center gap-2'>
                {stars !== null && (
                    <span className='inline-flex items-center gap-1'>
                        <Star className='size-3' />
                        {stars >= 1000 ? `${(stars / 1000).toFixed(1)}k` : stars}
                    </span>
                )}
                {forks !== null && (
                    <span className='inline-flex items-center gap-1'>
                        <GitFork className='size-3' />
                        {forks >= 1000 ? `${(forks / 1000).toFixed(1)}k` : forks}
                    </span>
                )}
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
