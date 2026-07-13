import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { CornerDownLeft, FileText, Search, X } from 'lucide-react';
import { getSearchIndex, type SearchIndexItem } from '@/lib/source';

type SearchResult = SearchIndexItem & {
  score: number;
  snippet: string;
};

const searchIndex = getSearchIndex();
const defaultResults = searchIndex
  .filter((item) => item.kind === 'page')
  .slice(0, 6)
  .map((item) => ({ ...item, score: 0, snippet: item.description }));

function normalize(value: string): string {
  return value.toLowerCase().replace(/\s+/g, ' ').trim();
}

function scoreResult(query: string, item: SearchIndexItem): number {
  const title = normalize(item.title);
  const description = normalize(item.description);
  const text = normalize(item.text);
  const parts = query.split(/\s+/).filter(Boolean);

  let score = 0;
  if (title === query) score += 40;
  if (title.startsWith(query)) score += 24;
  if (title.includes(query)) score += 16;
  if (description.includes(query)) score += 8;
  if (text.includes(query)) score += 4;
  if (item.kind === 'section') score += 3;

  for (const part of parts) {
    if (title.includes(part)) score += 8;
    if (description.includes(part)) score += 4;
    if (text.includes(part)) score += 2;
  }

  return score;
}

function createSnippet(query: string, item: SearchIndexItem): string {
  const source = item.text.replace(/\s+/g, ' ').trim();
  const normalizedSource = source.toLowerCase();
  const firstMatch = query
    .split(/\s+/)
    .filter(Boolean)
    .map((part) => normalizedSource.indexOf(part))
    .filter((index) => index >= 0)
    .sort((a, b) => a - b)[0];

  if (firstMatch === undefined) {
    return item.description || item.breadcrumbs.join(' / ');
  }

  const start = Math.max(0, firstMatch - 70);
  const end = Math.min(source.length, firstMatch + 170);
  const prefix = start > 0 ? '...' : '';
  const suffix = end < source.length ? '...' : '';

  return `${prefix}${source.slice(start, end)}${suffix}`;
}

function resultPath(result: SearchIndexItem): string {
  return [...result.breadcrumbs, result.title].filter(Boolean).join(' / ');
}

export default function DocsSearch() {
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const normalizedQuery = normalize(query);
  const hasQuery = normalizedQuery.length >= 2;

  const results = useMemo<SearchResult[]>(() => {
    if (!hasQuery) return defaultResults;

    return searchIndex
      .map((item) => ({
        ...item,
        score: scoreResult(normalizedQuery, item),
        snippet: createSnippet(normalizedQuery, item),
      }))
      .filter((item) => item.score > 0)
      .sort((a, b) => b.score - a.score || a.title.localeCompare(b.title))
      .slice(0, 12);
  }, [hasQuery, normalizedQuery]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const isSearchShortcut = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k';
      if (!isSearchShortcut) return;

      event.preventDefault();
      setOpen(true);
      setActiveIndex(0);
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  useEffect(() => {
    if (!open) return;

    const frame = window.requestAnimationFrame(() => inputRef.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, [open]);

  function closeSearch() {
    setOpen(false);
    setQuery('');
    setActiveIndex(0);
  }

  function openSearch() {
    setOpen(true);
    setActiveIndex(0);
  }

  function updateQuery(value: string) {
    setQuery(value);
    setActiveIndex(0);
  }

  function openResult(result: SearchResult) {
    navigate(result.url);
    closeSearch();
  }

  function handleSearchKeyDown(event: React.KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'Escape') {
      event.preventDefault();
      closeSearch();
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActiveIndex((index) => Math.min(index + 1, Math.max(results.length - 1, 0)));
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveIndex((index) => Math.max(index - 1, 0));
      return;
    }

    if (event.key === 'Enter' && results[activeIndex]) {
      event.preventDefault();
      openResult(results[activeIndex]);
    }
  }

  return (
    <>
      <div className="border-b border-fd-border bg-fd-background/95 px-4 py-3 backdrop-blur md:px-6">
        <div className="mx-auto max-w-3xl">
          <button
            type="button"
            onClick={openSearch}
            className="flex h-10 w-full items-center gap-3 rounded-md border border-fd-border bg-fd-muted/30 px-3 text-left text-sm text-fd-muted-foreground outline-none transition hover:bg-fd-muted/50 focus-visible:border-fd-primary focus-visible:ring-2 focus-visible:ring-fd-primary/20"
          >
            <Search className="size-4" />
            <span className="min-w-0 flex-1">Search API docs, guides, endpoints, schemas...</span>
            <kbd className="hidden rounded border border-fd-border bg-fd-background px-1.5 py-0.5 text-[11px] font-medium text-fd-muted-foreground sm:inline-flex">
              ⌘K
            </kbd>
          </button>
        </div>
      </div>

      {open ? (
        <div
          className="fixed inset-0 z-50 bg-black/40 px-4 py-16 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label="Search documentation"
          onMouseDown={closeSearch}
        >
          <div
            className="mx-auto max-w-2xl overflow-hidden rounded-lg border border-fd-border bg-fd-popover shadow-2xl"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <div className="flex items-center gap-3 border-b border-fd-border px-4">
              <Search className="size-4 text-fd-muted-foreground" />
              <input
                ref={inputRef}
                value={query}
                onChange={(event) => updateQuery(event.target.value)}
                onKeyDown={handleSearchKeyDown}
                placeholder="Search endpoint, schema, guide, or status code"
                className="h-14 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-fd-muted-foreground"
              />
              <button
                type="button"
                onClick={closeSearch}
                className="rounded-md p-1 text-fd-muted-foreground transition hover:bg-fd-muted hover:text-fd-foreground"
                aria-label="Close search"
              >
                <X className="size-4" />
              </button>
            </div>

            <div className="max-h-[60vh] overflow-y-auto p-2">
              {results.length > 0 ? (
                <div className="space-y-1">
                  {results.map((result, index) => (
                    <button
                      key={`${result.url}-${index}`}
                      type="button"
                      onMouseEnter={() => setActiveIndex(index)}
                      onClick={() => openResult(result)}
                      className={`flex w-full gap-3 rounded-md px-3 py-2.5 text-left transition ${
                        activeIndex === index ? 'bg-fd-accent text-fd-accent-foreground' : 'hover:bg-fd-accent/70'
                      }`}
                    >
                      <FileText className="mt-0.5 size-4 shrink-0 text-fd-muted-foreground" />
                      <span className="min-w-0 flex-1">
                        <span className="flex items-center gap-2">
                          <span className="truncate text-sm font-medium text-fd-foreground">{result.title}</span>
                          <span className="rounded border border-fd-border px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-fd-muted-foreground">
                            {result.kind}
                          </span>
                        </span>
                        <span className="mt-0.5 block truncate text-xs text-fd-muted-foreground">
                          {resultPath(result)}
                        </span>
                        <span className="mt-1 block line-clamp-2 text-xs text-fd-muted-foreground">
                          {result.snippet}
                        </span>
                      </span>
                      {activeIndex === index ? (
                        <CornerDownLeft className="mt-1 size-4 shrink-0 text-fd-muted-foreground" />
                      ) : null}
                    </button>
                  ))}
                </div>
              ) : (
                <div className="px-4 py-10 text-center text-sm text-fd-muted-foreground">
                  No documents found. Try a route, schema, service name, or error code.
                </div>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
