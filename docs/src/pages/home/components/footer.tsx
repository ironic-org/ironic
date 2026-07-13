import { Layers } from 'lucide-react';
import { Link } from 'react-router-dom';

const Footer = () => {
  return (
    <footer className="relative z-10 border-t border-fd-border bg-fd-background px-6 py-12">
      <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-8 md:flex-row">
        <div className="flex items-center gap-2">
          <span className="flex items-center justify-center rounded-md bg-brand/10 p-1 text-brand">
            <Layers className="h-4 w-4" />
          </span>
          <span className="text-sm font-bold text-fd-foreground">Ironic</span>
        </div>
        <div className="flex gap-8 text-[11px] font-bold uppercase tracking-widest text-fd-muted-foreground">
          <Link to="/docs" className="transition-colors hover:text-fd-foreground">
            Docs
          </Link>
          <a href="#features" className="transition-colors hover:text-fd-foreground">
            Workflows
          </a>
          <a href="#stack" className="transition-colors hover:text-fd-foreground">
            Stack
          </a>
        </div>
        <p className="text-[10px] font-bold uppercase tracking-widest text-fd-muted-foreground/60">
          Ironic {new Date().getFullYear()}.
        </p>
      </div>
    </footer>
  );
};

export default Footer;
