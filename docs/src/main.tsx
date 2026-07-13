import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider, createBrowserRouter } from 'react-router-dom';
import { appRoutes } from './routes';
import AppRoot from './AppRoot';
import './styles/global.css';

const router = createBrowserRouter([
  {
    element: <AppRoot />,
    children: appRoutes,
  },
]);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
