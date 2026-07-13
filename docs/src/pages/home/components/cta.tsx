import { Button } from '@/components/ui/button';
import { ArrowRight } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';

const CTA = () => {
    return (
        <section className='w-full relative py-32 overflow-hidden border-t border-fd-border bg-fd-background'>
            <div className='absolute inset-0 bg-linear-to-t from-brand/5 to-transparent pointer-events-none' />
            <div className='absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[400px] bg-brand/5 blur-[100px] rounded-full pointer-events-none' />

            <FadeUp className='relative max-w-4xl mx-auto px-6 flex flex-col items-center text-center z-10'>
                <h2 className='text-5xl md:text-7xl font-serif text-fd-foreground tracking-tight leading-[1.1] mb-8'>
                    Ready for{' '}
                    <span className='italic text-transparent bg-clip-text bg-linear-to-br from-brand to-fd-muted'>
                        frontend
                    </span>
                    <br /> integration?
                </h2>
                <p className='text-lg text-fd-muted-foreground font-medium max-w-xl mb-10'>
                    Start with the API Reference, then connect login, session,
                    and protected routes through one production API gateway.
                </p>
                <Button
                    asChild
                    className='h-14 px-10 rounded-full bg-brand text-brand-foreground font-bold text-lg hover:scale-105 transition-transform shadow-[0_0_30px_-5px_var(--color-brand)]'>
                    <Link to='/docs'>
                        Open API Reference
                        <ArrowRight className='ml-2 w-6 h-6' />
                    </Link>
                </Button>
            </FadeUp>
        </section>
    );
};

export default CTA;
