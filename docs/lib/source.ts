import type { Item, Node, Root } from 'fumadocs-core/page-tree';
import type { TOCItemType } from 'fumadocs-core/toc';
import type { ComponentType } from 'react';

type Frontmatter = {
  title?: string;
  description?: string;
  full?: boolean;
};

type CompiledDocModule = {
  default: ComponentType<{ components?: Record<string, unknown> }>;
  frontmatter?: Frontmatter;
  toc?: TOCItemType[];
  _markdown?: string;
};

type FolderMeta = {
  title?: string;
  description?: string;
  pages?: string[];
};

export type RuntimePageData = {
  title?: string;
  description?: string;
  body: ComponentType<{ components?: Record<string, unknown> }>;
  toc?: TOCItemType[];
  full?: boolean;
  searchText?: string;
  getText?: (mode: 'processed') => Promise<string>;
};

export type RuntimePage = {
  url: string;
  slugs: string[];
  data: RuntimePageData;
};

export type SearchIndexItem = {
  title: string;
  description: string;
  url: string;
  text: string;
  breadcrumbs: string[];
  kind: 'page' | 'section';
};

const docModules = import.meta.glob('../content/docs/**/*.{md,mdx}', {
  eager: true,
  query: {
    collection: 'docs',
  },
}) as Record<string, CompiledDocModule>;

const blogModules = import.meta.glob('../content/blog/**/*.{md,mdx}', {
  eager: true,
  query: {
    collection: 'docs',
  },
}) as Record<string, CompiledDocModule>;

const metaModules = import.meta.glob('../content/docs/**/meta.json', {
  eager: true,
  import: 'default',
}) as Record<string, FolderMeta>;

function toSlugs(filePath: string): string[] {
  const relative = filePath.replace('../content/docs/', '').replace(/\.(md|mdx)$/, '');
  if (relative === 'index') return [];
  if (relative.endsWith('/index')) return relative.split('/').slice(0, -1);
  return relative.split('/');
}

function toUrl(slugs: string[]): string {
  return slugs.length === 0 ? '/docs' : `/docs/${slugs.join('/')}`;
}

function createPage(module: CompiledDocModule, slugs: string[]): RuntimePage {
  return {
    url: toUrl(slugs),
    slugs,
    data: {
      title: module.frontmatter?.title,
      description: module.frontmatter?.description,
      body: module.default,
      toc: module.toc ?? [],
      full: module.frontmatter?.full,
      searchText: module._markdown ?? '',
      getText: async () => module._markdown ?? '',
    },
  };
}

const pages = Object.entries(docModules)
  .map(([filePath, module]) => createPage(module, toSlugs(filePath)))
  .sort((a, b) => a.url.localeCompare(b.url));

const pageMap = new Map(pages.map((page) => [page.slugs.join('/'), page]));
const folderMetas = new Map<string, FolderMeta>(
  Object.entries(metaModules).map(([filePath, meta]) => [
    filePath.replace('../content/docs/', '').replace(/\/?meta\.json$/, ''),
    meta,
  ]),
);

function titleCase(value: string): string {
  return value
    .split(/[._\-\s]+/)
    .filter(Boolean)
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(' ');
}

function pageKey(slugs: string[]): string {
  return slugs.join('/');
}

function pageTitle(slugs: string[], fallback: string): string {
  return pageMap.get(pageKey(slugs))?.data.title ?? folderMetas.get(pageKey(slugs))?.title ?? titleCase(fallback);
}

function pageDescription(slugs: string[]): string | undefined {
  return pageMap.get(pageKey(slugs))?.data.description ?? folderMetas.get(pageKey(slugs))?.description;
}

function pageNode(slugs: string[], fallback: string): Item {
  return {
    type: 'page',
    name: pageTitle(slugs, fallback),
    url: toUrl(slugs),
    description: pageDescription(slugs),
  };
}

function isSeparator(entry: string): boolean {
  return entry.startsWith('---') && entry.endsWith('---');
}

function separatorName(entry: string): string {
  return entry.replace(/^-+|-+$/g, '').trim();
}

function folderNode(slugs: string[], fallback: string): Node {
  const meta = folderMetas.get(pageKey(slugs));
  const indexSlugs = [...slugs, 'index'];
  const hasIndex = pageMap.has(pageKey(slugs));

  return {
    type: 'folder',
    name: meta?.title ?? pageTitle(slugs, fallback),
    description: meta?.description,
    defaultOpen: true,
    index: hasIndex ? pageNode(slugs, 'Overview') : undefined,
    children: treeChildren(slugs).filter((node) => {
      if (node.type !== 'page') return true;
      return node.url !== toUrl(indexSlugs.slice(0, -1));
    }),
  };
}

function treeChildren(parentSlugs: string[]): Node[] {
  const meta = folderMetas.get(pageKey(parentSlugs));
  const entries = meta?.pages ?? [];

  return entries.flatMap((entry): Node[] => {
    if (isSeparator(entry)) {
      return [{ type: 'separator', name: separatorName(entry) }];
    }

    if (entry === 'index') {
      return [pageNode(parentSlugs, 'Overview')];
    }

    const slugs = [...parentSlugs, entry];
    if (folderMetas.has(pageKey(slugs))) {
      return [folderNode(slugs, entry)];
    }

    if (pageMap.has(pageKey(slugs))) {
      return [pageNode(slugs, entry)];
    }

    return [];
  });
}

export const source = {
  pageTree: {
    name: 'Docs',
    children: treeChildren([]),
  } as Root,
  getPage(slugs?: string[]) {
    return pageMap.get((slugs ?? []).join('/'));
  },
  getPages() {
    return pages;
  },
  getNodeMeta() {
    return null;
  },
};

export function getRuntimePageData(page: RuntimePage): RuntimePageData {
  return page.data;
}

export function getPageImage(page: RuntimePage) {
  const segments = [...page.slugs, 'image.png'];

  return {
    segments,
    url: '',
  };
}

export async function getLLMText(page: RuntimePage) {
  const processed = await page.data.getText?.('processed');

  return `# ${page.data.title ?? 'Untitled'}

${processed ?? ''}`;
}

export function getSearchIndex(): SearchIndexItem[] {
  return pages.flatMap((page) => {
    const pageTitleValue = page.data.title ?? 'Untitled';
    const pageDescriptionValue = page.data.description ?? '';
    const pageText = page.data.searchText ?? '';
    const pageItem: SearchIndexItem = {
      title: pageTitleValue,
      description: pageDescriptionValue,
      url: page.url,
      text: `${pageTitleValue}\n${pageDescriptionValue}\n${pageText}`,
      breadcrumbs: page.slugs.length > 0 ? page.slugs.slice(0, -1).map(titleCase) : ['Docs'],
      kind: 'page',
    };

    const sectionItems = (page.data.toc ?? [])
      .filter((item) => typeof item.title === 'string' && item.url)
      .map((item): SearchIndexItem => ({
        title: String(item.title),
        description: pageTitleValue,
        url: `${page.url}${item.url}`,
        text: `${item.title}\n${pageTitleValue}\n${pageDescriptionValue}\n${pageText}`,
        breadcrumbs: [...(page.slugs.length > 0 ? page.slugs.map(titleCase) : ['Docs'])],
        kind: 'section',
      }));

    return [pageItem, ...sectionItems];
  });
}

// ── Blog source ──────────────────────────────────────────

const blogPages = Object.entries(blogModules)
  .map(([filePath, module]) => {
    const relative = filePath.replace('../content/blog/', '').replace(/\.(md|mdx)$/, '');
    const slugs = relative === 'index' ? [] : relative.split('/');
    return createPage(module, slugs);
  });

const blogPageMap = new Map(blogPages.map((page) => [page.slugs.join('/'), page]));

export const blogSource = {
  getPage(slugs?: string[]) {
    return blogPageMap.get((slugs ?? []).join('/'));
  },
  getPages() {
    return blogPages;
  },
};
