import { GitFork, Star } from 'lucide-react';
import { useEffect, useState } from 'react';

function formatCount(n: number): string {
    if (n >= 1000) {
        return `${(n / 1000).toFixed(1)}k`;
    }
    return n.toString();
}

export function useGitHubStars() {
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

    return { stars, forks };
}

export function GitHubStatsBadge({ className }: { className?: string }) {
    const { stars, forks } = useGitHubStars();

    return (
        <div className={`inline-flex items-center gap-0.5 rounded-full border border-fd-border bg-fd-card/60 px-0.5 py-0.5 text-xs font-medium ${className ?? ''}`}>
            <div className='inline-flex items-center gap-1 rounded-full bg-gradient-to-br from-amber-400/15 to-yellow-500/10 px-2 py-1'>
                <Star className='size-3 fill-amber-400 stroke-amber-500' />
                <span className='text-amber-900 dark:text-amber-200 tabular-nums'>
                    {stars !== null ? formatCount(stars) : '—'}
                </span>
            </div>
            <div className='inline-flex items-center gap-1 rounded-full bg-gradient-to-br from-sky-400/10 to-blue-500/10 px-2 py-1'>
                <GitFork className='size-3 stroke-sky-500' />
                <span className='text-sky-700 dark:text-sky-300 tabular-nums'>
                    {forks !== null ? formatCount(forks) : '—'}
                </span>
            </div>
        </div>
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
