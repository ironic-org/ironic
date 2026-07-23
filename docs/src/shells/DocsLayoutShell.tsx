import type { ReactNode } from 'react';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';
import { source } from '@/lib/source';
import DocsSearch from '../components/DocsSearch';
import { GithubSidebarFooter } from '../components/github-sidebar';

export default function DocsLayoutShell({ children }: { children: ReactNode }) {
  return (
    <DocsLayout
      tree={source.pageTree}
      {...baseOptions()}
      sidebar={{
        tabs: false,
        footer: <GithubSidebarFooter />,
      }}
      themeSwitch={{ enabled: false }}
    >
      <DocsSearch />
      {children}
    </DocsLayout>
  );
}
