import { Button } from '@/components/ui/button';
import { ChevronRight, Sparkles } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';
import MarqueeContent from './marquee-content';

const HeroSection = () => {
    return (
        <section className='relative pt-24 pb-20 px-6 max-w-7xl mx-auto flex flex-col items-center text-center'>
            <FadeUp>
                <a
                    href="/docs"
                    className='group relative inline-flex items-center gap-3 rounded-full border border-fd-border bg-fd-card/50 pr-4 pl-1.5 py-1.5 hover:border-fd-accent transition-all mb-10'>
                    <span className='rounded-full bg-brand/10 border border-brand/20 px-2.5 py-0.5 text-[10px] font-bold text-brand tracking-widest uppercase'>
                        V0.1 PREVIEW
                    </span>
                    <span className='text-xs font-medium text-fd-muted-foreground flex items-center gap-1 group-hover:text-fd-foreground transition-colors'>
                        RustFrame
                        <ChevronRight className='w-3 h-3 text-fd-muted-foreground/60' />
                    </span>
                </a>
            </FadeUp>

            <FadeUp delay='100ms'>
                <h1 className='text-5xl sm:text-7xl md:text-8xl font-bold tracking-tighter text-fd-foreground leading-[1.05] mb-6'>
                    Build, deploy,
                    <br />
                    <span className='font-serif italic font-normal text-brand inline-block mt-2'>
                        and operate.
                    </span>
                </h1>
            </FadeUp>

            <FadeUp delay='200ms' className='max-w-2xl mx-auto'>
                <p className='text-lg md:text-xl text-fd-muted-foreground font-medium leading-relaxed mb-12'>
                    A modular, type-safe application framework for structured
                    Rust APIs on Axum.
                </p>
            </FadeUp>

            <FadeUp
                delay='300ms'
                className='flex flex-col sm:flex-row gap-4 justify-center items-center w-full'>
                <Button
                    asChild
                    className='group h-14 px-12 rounded-full bg-brand hover:opacity-90 text-brand-foreground font-bold text-md shadow-[0_0_20px_-5px_var(--color-brand)] transition-all'>
                    <Link to='/docs'>
                        Read the Docs
                        <Sparkles className='ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform' />
                    </Link>
                </Button>

            </FadeUp>

            {/* Logo Cloud / Tech Stack Marquee */}
            <FadeUp delay='400ms' className='mt-24 w-full'>
                <p className='text-[10px] font-bold text-fd-muted-foreground/60 uppercase tracking-[0.3em] mb-10'>
                    Built on explicit Rust contracts
                </p>
                <div className='relative flex overflow-hidden group'>
                    <MarqueeContent />
                </div>
            </FadeUp>
        </section>
    );
};

export default HeroSection;
