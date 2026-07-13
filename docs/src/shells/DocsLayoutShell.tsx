import type { ReactNode } from 'react';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';
import { source } from '@/lib/source';
import DocsSearch from '../components/DocsSearch';

export default function DocsLayoutShell({ children }: { children: ReactNode }) {
  return (
    <DocsLayout
      tree={source.pageTree}
      {...baseOptions()}
      sidebar={{
        tabs: false,
      }}
    >
      <DocsSearch />
      {children}
    </DocsLayout>
  );
}
