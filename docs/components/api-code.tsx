'use client';

import { Children, type ReactNode, isValidElement, useState } from 'react';
import type { ReactElement } from 'react';
import { cn } from '@/lib/utils';

type CodeLang = 'curl' | 'js' | 'leptos' | 'kotlin';

interface ApiCodeProps {
  className?: string;
  children: ReactNode;
  langs?: CodeLang[];
}

interface ApiCodeTabProps {
  lang: CodeLang;
  children: ReactNode;
}

export function ApiCodeTab({ children }: ApiCodeTabProps) {
  return <>{children}</>;
}

export function ApiCode({ className, children, langs = ['curl', 'js', 'leptos', 'kotlin'] }: ApiCodeProps) {
  const tabs = Children.toArray(children).filter(
    (child): child is ReactElement<ApiCodeTabProps> =>
      isValidElement(child) && child.type === ApiCodeTab,
  );

  const [activeIndex, setActiveIndex] = useState(0);
  const active = tabs[activeIndex];

  const displayLangs = tabs.length > 0
    ? tabs.map((t) => t.props.lang)
    : langs;

  const labelMap: Record<CodeLang, string> = {
    curl: 'cURL',
    js: 'JavaScript',
    leptos: 'Leptos',
    kotlin: 'Kotlin',
  };

  return (
    <div className={cn('my-4 rounded-lg border border-fd-border overflow-hidden', className)}>
      <div className="flex gap-0 bg-fd-muted/30 border-b border-fd-border">
        {displayLangs.map((lang, i) => (
          <button
            key={lang}
            type="button"
            onClick={() => setActiveIndex(i)}
            className={cn(
              'px-4 py-2 text-xs font-medium transition-colors border-b-2 -mb-px',
              i === activeIndex
                ? 'border-brand text-brand bg-fd-background'
                : 'border-transparent text-fd-muted-foreground hover:text-fd-foreground',
            )}
          >
            {labelMap[lang]}
          </button>
        ))}
      </div>
      <div className="bg-fd-card [&_pre]:!mt-0 [&_pre]:!rounded-none [&_pre]:!border-0">
        {active}
      </div>
    </div>
  );
}
