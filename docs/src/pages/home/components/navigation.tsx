import { Github, Layers } from 'lucide-react';
import { Link } from 'react-router-dom';

const Navigation = () => {
    return (
        <nav className='sticky top-0 z-50 w-full border-b border-fd-border bg-fd-background/80 backdrop-blur-md'>
            <div className='mx-auto flex h-16 max-w-7xl items-center justify-between px-6'>
                <Link to='/' className='flex items-center gap-2'>
                    <span className='flex items-center justify-center rounded-lg border border-brand/20 bg-brand/10 p-1.5 text-brand'>
                        <Layers className='h-5 w-5' />
                    </span>
                    <span className='text-lg font-bold tracking-tight text-fd-foreground'>
                        Ironic
                    </span>
                </Link>

                <div className='hidden items-center gap-8 md:flex'>
                    <a
                        href='#features'
                        className='text-sm font-medium text-fd-muted-foreground transition-colors hover:text-brand'>
                        Features
                    </a>
                    <Link
                        to='/docs'
                        className='text-sm font-medium text-fd-muted-foreground transition-colors hover:text-brand'>
                        Documentation
                    </Link>
                    <Link
                        to='/docs/examples'
                        className='text-sm font-medium text-fd-muted-foreground transition-colors hover:text-brand'>
                        Examples
                    </Link>
                    <a
                        href='https://github.com/ironic-org/ironic'
                        target='_blank'
                        rel='noopener noreferrer'
                        className='text-fd-muted-foreground hover:text-fd-foreground transition-colors'>
                        <Github className='h-5 w-5' />
                    </a>
                </div>
            </div>
        </nav>
    );
};

export default Navigation;
