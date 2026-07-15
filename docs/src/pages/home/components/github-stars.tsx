import { Star } from 'lucide-react';
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

export function StarCount({ className }: { className?: string }) {
    const { stars } = useGitHubStars();
    if (stars === null) return null;
    return (
        <span className={className}>
            <Star className='inline size-3 mr-1' />
            {formatCount(stars)}
        </span>
    );
}
