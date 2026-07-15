import { Button } from '@/components/ui/button';
import { ArrowRight, Github, Terminal } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';
import { GitHubStarButton } from './github-stars';

const CTA = () => {
    return (
        <section className='w-full relative py-24 overflow-hidden border-t border-fd-border'>
            <div className='absolute inset-0 bg-linear-to-b from-brand/4 via-transparent to-transparent pointer-events-none' />
            <div className='absolute -top-40 left-1/2 -translate-x-1/2 w-175 h-100 bg-brand/4 rounded-full blur-[120px] pointer-events-none' />

            <FadeUp className='relative max-w-2xl mx-auto px-6 flex flex-col items-center text-center z-10'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-4'>
                    Start building today
                </h2>
                <p className='text-base text-fd-muted-foreground font-medium mb-3'>
                    Install the CLI, scaffold a project, and have a running
                    API in under 60 seconds.
                </p>
                <div className='inline-flex items-center gap-2 rounded-full border border-fd-border bg-fd-card/50 px-4 py-2 text-xs font-mono text-fd-muted-foreground mb-8'>
                    <Terminal className='w-3.5 h-3.5 text-brand' />
                    <span>cargo install ironic</span>
                </div>
                <div className='flex flex-col sm:flex-row gap-3'>
                    <Button
                        asChild
                        size='lg'
                        className='h-12 px-8 rounded-full bg-brand hover:bg-brand/90 text-white font-semibold text-sm shadow-lg shadow-brand/20 transition-all hover:shadow-brand/30 hover:scale-[1.02]'>
                        <Link to='/docs/getting-started'>
                            Get started
                            <ArrowRight className='ml-2 w-4 h-4' />
                        </Link>
                    </Button>
                    <Button
                        asChild
                        variant='outline'
                        size='lg'
                        className='h-12 px-8 rounded-full border-fd-border hover:bg-fd-accent font-semibold text-sm transition-all'>
                        <a
                            href='https://github.com/ironic-org/ironic'
                            target='_blank'
                            rel='noopener noreferrer'
                            className='inline-flex items-center'>
                            <Github className='mr-2 w-4 h-4' />
                            Star on GitHub
                            <GitHubStarButton />
                        </a>
                    </Button>
                </div>
            </FadeUp>
        </section>
    );
};

export default CTA;
