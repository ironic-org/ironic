# Ironic documentation site

The documentation site uses Vite, React Router, Fumadocs, and Tailwind CSS.

```bash
bun run dev
bun run check-types
bun run build
```

`bun run build` compiles the site, copies `index.html` to `404.html`, and generates `dist/sitemap.xml` (via `scripts/generate-sitemap.mjs`) so `robots.txt`'s sitemap reference resolves.

## Content structure

Framework documentation lives under `content/docs`. `meta.json` controls sidebar ordering within each category. Engineering blog posts live under `content/blog` (each is a Markdown file with `title`, `description`, `date`, and `author` frontmatter).

```
content/docs/
├── meta.json                  # Top-level category ordering
├── getting-started/
├── project-structure/
├── fundamentals/              # Concepts: modules, providers, DI, pipes, request lifecycle
├── core/                      # Internals: DI resolution, lifetimes, macros, container overrides
├── lifecycle/                 # Lifecycle hooks
├── configuration/
├── modules/                   # Advanced module patterns: dynamic modules, decorators
├── http-api/                  # Routes, middleware, guards, OpenAPI, security, etc.
├── transport/                 # HTTP, WebSocket, GraphQL, MCP, SSE, events
├── distributed/               # Microservices, queues, sagas, events, outbox
├── middleware/
├── data-auth/
├── performance/               # Caching, scheduling
├── observability/             # Metrics, tracing, health checks
├── testing/
├── advanced/
├── quick-learn/               # Feature reference
├── releases/                  # Version history + per-series changelogs
├── migrations/
└── more/                      # Deployment, FAQ, examples, benchmarks
```

## Writing guidelines

Every documentation page follows this structure:

```markdown
---
title: Page Title
description: Compelling one-line summary of what this page covers.
---

# Page Title

## What you'll learn

- Bullet list of 3-5 learning objectives

Intro paragraph. Use `> **Why this matters:**` callout boxes for motivation.

---

## Section Name

Explanation + code example. Use ASCII diagrams where helpful.

---

## Try it yourself

1. Step-by-step exercise
2. ...
3. ...

## Common mistakes

| Mistake | Fix |
|---------|-----|
| ...     | ... |

## What you learned

- [x] Key takeaway
- [x] Key takeaway
```

### Style rules

- **Code examples**: Always complete and compilable. Use `ironic::prelude::*` imports.
- **Tables**: Prefer tables over lists for comparison/API reference content.
- **Callouts**: Use `> **Note:**` for caveats, `> **Why this matters:**` for motivation.
- **Feature flags**: Always mention if a feature requires a Cargo.toml feature flag.
- **Word count**: Target 400-600 words for dedicated pages; 200-300 for sub-pages within a category.

## Contributing

1. Create or edit Markdown files under `content/docs/`
2. Update the relevant `meta.json` to add new pages to the sidebar
3. Run `bun run dev` to preview locally at http://localhost:3002
4. Verify all code examples compile correctly
