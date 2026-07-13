import { Navigate, type RouteObject } from 'react-router-dom';
import DocsPage from './pages/DocsPage';
import HomePage from './pages/HomePage';

export const appRoutes: RouteObject[] = [
  { path: '/', element: <HomePage /> },
  { path: '/docs', element: <DocsPage /> },
  { path: '/docs/*', element: <DocsPage /> },
  { path: '*', element: <Navigate to="/" replace /> },
];
