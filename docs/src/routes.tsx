import { lazy, Suspense } from 'react';
import { Navigate, type RouteObject } from 'react-router-dom';

const HomePage = lazy(() => import('./pages/HomePage'));
const DocsPage = lazy(() => import('./pages/DocsPage'));

function withSuspense(node: React.ReactNode): React.ReactNode {
  return <Suspense fallback={<div />}>{node}</Suspense>;
}

export const appRoutes: RouteObject[] = [
  { path: '/', element: withSuspense(<HomePage />) },
  { path: '/docs', element: withSuspense(<DocsPage />) },
  { path: '/docs/*', element: withSuspense(<DocsPage />) },
  { path: '*', element: <Navigate to="/" replace /> },
];
