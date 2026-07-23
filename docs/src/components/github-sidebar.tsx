import { Github, Star, GitFork } from 'lucide-react';
import { useEffect, useState } from 'react';

function formatCount(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return n.toString();
}

export function GithubSidebarFooter() {
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
      className='flex items-center gap-2 rounded-lg border border-fd-border bg-fd-card/50 px-3 py-2 text-sm text-fd-muted-foreground hover:bg-fd-accent hover:text-fd-accent-foreground transition-colors mx-2 mb-2'
    >
      <Github className='size-4 shrink-0' />
      <span className='flex-1 truncate font-medium'>ironic-org/ironic</span>
      <span className='inline-flex items-center gap-1 rounded-full bg-amber-400/10 px-1.5 py-0.5 text-xs'>
        <Star className='size-3 fill-amber-400 stroke-amber-500' />
        <span className='tabular-nums'>
          {stars !== null ? formatCount(stars) : '—'}
        </span>
      </span>
      <span className='inline-flex items-center gap-1 rounded-full bg-sky-400/10 px-1.5 py-0.5 text-xs'>
        <GitFork className='size-3 stroke-sky-500' />
        <span className='tabular-nums'>
          {forks !== null ? formatCount(forks) : '—'}
        </span>
      </span>
    </a>
  );
}
