import { useEffect, useMemo } from 'react';
import { useLocation } from 'react-router-dom';
import { getPageImage, getRuntimePageData, source } from '@/lib/source';
import { getMDXComponents } from '@/mdx-components';
import { DocsBody, DocsPage as FumaDocsPage } from 'fumadocs-ui/layouts/docs/page';
import DocsLayoutShell from '../shells/DocsLayoutShell';

function getSlug(pathname: string): string[] | undefined {
  const stripped = pathname.replace(/^\/docs\/?/, '').replace(/\/$/, '');
  if (!stripped) return undefined;
  return stripped.split('/').filter(Boolean);
}

export default function DocsPage() {
  const location = useLocation();
  const slug = useMemo(() => getSlug(location.pathname), [location.pathname]);
  const page = source.getPage(slug);
  const pageData = page ? getRuntimePageData(page) : null;

  useEffect(() => {
    if (!page || !pageData) {
      document.title = 'Page Not Found | RustFrame';
      return;
    }

    document.title = `${pageData.title ?? 'Untitled'} | RustFrame`;

    const description = pageData.description ?? '';
    const image = getPageImage(page).url;
    const setMeta = (name: string, content: string, attribute: 'name' | 'property' = 'name') => {
      let element = document.head.querySelector<HTMLMetaElement>(`meta[${attribute}="${name}"]`);
      if (!element) {
        element = document.createElement('meta');
        element.setAttribute(attribute, name);
        document.head.appendChild(element);
      }
      element.content = content;
    };

    setMeta('description', description);
    setMeta('og:title', pageData.title ?? 'Untitled', 'property');
    setMeta('og:description', description, 'property');
    setMeta('og:image', image, 'property');
  }, [page, pageData]);

  return (
    <DocsLayoutShell>
      {!page ? (
        <FumaDocsPage>
          <h1 className="text-[1.75em] font-semibold">Page not found</h1>
          <p className="mb-4 text-lg text-fd-muted-foreground">
            The requested document does not exist.
          </p>
        </FumaDocsPage>
      ) : (
        <DocContent />
      )}
    </DocsLayoutShell>
  );

  function DocContent() {
    if (!page) return null;

    const MDX = pageData?.body;
    if (!MDX) return null;

    return (
      <FumaDocsPage
        toc={pageData.toc}
        full={pageData.full}
        tableOfContent={{
          style: 'clerk',
        }}
      >
        <h1 className="text-[1.75em] font-semibold">{pageData.title ?? 'Untitled'}</h1>
        <p className="mb-4 text-lg text-fd-muted-foreground">{pageData.description}</p>
        <DocsBody>
          <MDX components={getMDXComponents()} />
        </DocsBody>
      </FumaDocsPage>
    );
  }
}
