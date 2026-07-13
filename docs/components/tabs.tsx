import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

function TabButton({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'px-3 py-1.5 text-xs font-medium rounded-md transition-colors',
        active
          ? 'bg-brand text-brand-foreground'
          : 'text-fd-muted-foreground hover:text-fd-foreground hover:bg-fd-muted',
      )}
    >
      {label}
    </button>
  );
}

export function Tabs({ items, className }: { items: { label: string; content: ReactNode }[]; className?: string }) {
  return (
    <div className={cn('my-4 rounded-lg border border-fd-border', className)}>
      <div className="flex gap-1 px-3 pt-3 pb-2 bg-fd-muted/50 rounded-t-lg border-b border-fd-border">
        {items.map((item, i) => (
          <TabButton key={i} label={item.label} active={i === 0} onClick={() => {}} />
        ))}
      </div>
      <div className="p-3 bg-fd-card rounded-b-lg">
        {items[0]?.content}
      </div>
    </div>
  );
}
