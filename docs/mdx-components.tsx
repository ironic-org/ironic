import defaultMdxComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';
import { ApiCode, ApiCodeTab } from '@/components/api-code';

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    ApiCode,
    ApiCodeTab,
    ...components,
  };
}
