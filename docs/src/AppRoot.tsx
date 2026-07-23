import { Outlet } from 'react-router-dom';
import { RootProvider } from 'fumadocs-ui/provider/react-router';
import { ThemeProvider } from 'next-themes';
import { docsThemeStyles } from '@/lib/theme';

export default function AppRoot() {
  return (
    <>
      <style>{docsThemeStyles}</style>
      <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
        <RootProvider>
          <Outlet />
        </RootProvider>
      </ThemeProvider>
    </>
  );
}
