import { Button } from '@/components/ui/button';
import { ArrowRight, Github } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';

const CTA = () => {
    return (
        <section className='w-full relative py-20 overflow-hidden border-t border-fd-border'>
            <div className='absolute inset-0 bg-linear-to-b from-brand/3 to-transparent pointer-events-none' />

            <FadeUp className='relative max-w-2xl mx-auto px-6 flex flex-col items-center text-center z-10'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-4'>
                    Start building today
                </h2>
                <p className='text-base text-fd-muted-foreground font-medium mb-8'>
                    Install the CLI, scaffold a project, and have a running
                    API in under 60 seconds.
                </p>
                <div className='flex flex-col sm:flex-row gap-3'>
                    <Button
                        asChild
                        size='lg'
                        className='h-12 px-8 rounded-full bg-brand hover:bg-brand/90 text-white font-semibold text-sm shadow-lg shadow-brand/20 transition-all'>
                        <Link to='/docs/getting-started/getting-started'>
                            Get started
                            <ArrowRight className='ml-2 w-4 h-4' />
                        </Link>
                    </Button>
                    <Button
                        asChild
                        variant='outline'
                        size='lg'
                        className='h-12 px-8 rounded-full border-fd-border hover:bg-fd-accent font-semibold text-sm transition-all'>
                        <a href='https://github.com/ironic-org/ironic' target='_blank' rel='noopener noreferrer'>
                            <Github className='mr-2 w-4 h-4' />
                            Star on GitHub
                        </a>
                    </Button>
                </div>
            </FadeUp>
        </section>
    );
};

export default CTA;
