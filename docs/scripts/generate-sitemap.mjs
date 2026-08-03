import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const root = new URL('..', import.meta.url).pathname;
const docsContent = join(root, 'content/docs');
const blogContent = join(root, 'content/blog');
const distDir = join(root, 'dist');

const SITE_URL = (process.env.SITE_URL || 'https://ironic-org.github.io/ironic').replace(/\/$/, '');
const today = new Date().toISOString().slice(0, 10);

function collectFiles(dir, base = '') {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'meta.json') continue;
    if (entry.name.startsWith('.')) continue;
    const rel = base ? `${base}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      out.push(...collectFiles(join(dir, entry.name), rel));
    } else if (entry.name.endsWith('.md') || entry.name.endsWith('.mdx')) {
      out.push(rel);
    }
  }
  return out;
}

function toUrl(rel, isBlog) {
  const stripped = rel.replace(/\.(md|mdx)$/, '');
  const path = stripped === 'index' ? '' : stripped.endsWith('/index') ? stripped.slice(0, -6) : stripped;
  return `${SITE_URL}/${isBlog ? 'blog/' : 'docs/'}${path}`;
}

const docsUrls = collectFiles(docsContent).map((rel) => toUrl(rel, false));
const blogUrls = collectFiles(blogContent).map((rel) => toUrl(rel, true));
const staticUrls = [`${SITE_URL}/`, `${SITE_URL}/blog`];

const urls = [...new Set([...staticUrls, ...docsUrls, ...blogUrls])].sort();

const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.map((loc) => `  <url>\n    <loc>${loc}</loc>\n    <lastmod>${today}</lastmod>\n  </url>`).join('\n')}
</urlset>
`;

writeFileSync(join(distDir, 'sitemap.xml'), xml);
console.log(`sitemap.xml generated: ${urls.length} URLs`);
