import { GitFork, Star, Github } from 'lucide-react';
import { useEffect, useState } from 'react';
import { GITHUB_API_URL, GITHUB_URL } from '@/lib/constants';

function formatCount(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
    return n.toString();
}

type GitHubData = { stars: number | null; forks: number | null };

const cache = { stars: null as number | null, forks: null as number | null };

export function useGitHubStars(): GitHubData {
    const [data, setData] = useState<GitHubData>({ stars: cache.stars, forks: cache.forks });

    useEffect(() => {
        if (cache.stars !== null && cache.forks !== null) return;
        fetch(GITHUB_API_URL)
            .then((res) => res.json())
            .then((d) => {
                const s = d.stargazers_count ?? null;
                const f = d.forks_count ?? null;
                cache.stars = s;
                cache.forks = f;
                setData({ stars: s, forks: f });
            })
            .catch(() => { });
    }, []);

    return data;
}

function Pill({ icon, count, bg, text, iconClass }: {
    icon: React.ReactNode;
    count: number | null;
    bg: string;
    text: string;
    iconClass: string;
}) {
    return (
        <span className={`inline-flex items-center gap-1 rounded-full ${bg} px-2 py-0.5`}>
            <span className={iconClass}>{icon}</span>
            <span className={`text-xs font-semibold tabular-nums ${text}`}>
                {count !== null ? formatCount(count) : '—'}
            </span>
        </span>
    );
}

export function GitHubStatsBadge({ className, showFork = true }: { className?: string; showFork?: boolean }) {
    const { stars, forks } = useGitHubStars();

    return (
        <span className={`inline-flex items-center gap-1 rounded-full border border-fd-border bg-fd-card/60 px-1 py-1 text-xs font-medium ${className ?? ''}`}>
            <Pill
                icon={<Star className='size-3' />}
                count={stars}
                bg='bg-linear-to-br from-amber-400/15 to-yellow-500/10'
                text='text-amber-800 dark:text-amber-200'
                iconClass='fill-amber-400 stroke-amber-500'
            />
            {showFork && (
                <Pill
                    icon={<GitFork className='size-3' />}
                    count={forks}
                    bg='bg-linear-to-br from-sky-400/10 to-blue-500/10'
                    text='text-sky-700 dark:text-sky-300'
                    iconClass='stroke-sky-500'
                />
            )}
        </span>
    );
}

export function GitHubNavBadge({ className }: { className?: string }) {
    return (
        <a
            href={GITHUB_URL}
            target='_blank'
            rel='noopener noreferrer'
            className={`inline-flex items-center gap-1.5 text-fd-muted-foreground hover:text-fd-foreground transition-colors group ${className ?? ''}`}
        >
            <Github className='size-5 group-hover:scale-110 transition-transform' />
            <GitHubStatsBadge />
        </a>
    );
}

export function GitHubStarButton() {
    const { stars } = useGitHubStars();

    return (
        <span className='inline-flex items-center gap-1.5'>
            {stars !== null ? (
                <span className='inline-flex items-center gap-1 rounded-full bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 px-2 py-0.5'>
                    <Star className='size-3 fill-amber-400 stroke-amber-500' />
                    <span className='text-xs font-semibold text-amber-700 dark:text-amber-300 tabular-nums'>
                        {formatCount(stars)}
                    </span>
                </span>
            ) : (
                <Star className='size-3 fill-amber-400 stroke-amber-500' />
            )}
        </span>
    );
}

export function StarCount({ className }: { className?: string }) {
    const { stars } = useGitHubStars();
    if (stars === null) return null;
    return (
        <span className={className}>
            <Star className='inline size-3 fill-amber-400 stroke-amber-500 mr-1' />
            {formatCount(stars)}
        </span>
    );
}
