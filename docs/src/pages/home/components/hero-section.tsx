import { Button } from '@/components/ui/button';
import { ArrowRight, Github } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';
import { GitHubStarButton } from './github-stars';

const HeroSection = () => {
    return (
        <section className='relative min-h-[100svh] overflow-hidden'>
            <div className='absolute inset-0 pointer-events-none' aria-hidden='true'>
                <div className='absolute inset-0 bg-[radial-gradient(ellipse_at_20%_0%,rgba(224,120,64,0.18),transparent_45%),radial-gradient(ellipse_at_85%_15%,rgba(125,211,192,0.12),transparent_40%),linear-gradient(180deg,rgba(11,16,26,0.04),transparent_35%,var(--color-fd-background))]' />
                <div className='absolute -top-24 left-1/2 h-[42rem] w-[42rem] -translate-x-1/2 rounded-full bg-brand/10 blur-[100px] animate-drift' />
                <div className='absolute bottom-0 right-0 h-[28rem] w-[28rem] translate-x-1/4 translate-y-1/4 rounded-full bg-[rgba(125,211,192,0.08)] blur-[90px] animate-drift' style={{ animationDelay: '4s' }} />
                <svg
                    className='absolute inset-x-0 bottom-0 h-[55%] w-full opacity-[0.55] dark:opacity-70'
                    viewBox='0 0 1200 520'
                    fill='none'
                    preserveAspectRatio='xMidYMax slice'>
                    <path d='M120 120H420M420 120V260M420 260H780M780 260V400M780 400H1080' stroke='currentColor' className='text-fd-border' strokeWidth='1.5' />
                    <path d='M220 400H520M520 400V200M520 200H900' stroke='currentColor' className='text-brand/25' strokeWidth='1.5' />
                    <circle className='animate-node-pulse' cx='120' cy='120' r='7' fill='#7DD3C0' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '0.6s' }} cx='420' cy='120' r='7' fill='#E07840' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '1.2s' }} cx='420' cy='260' r='7' fill='#7DD3C0' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '1.8s' }} cx='780' cy='260' r='7' fill='#E07840' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '2.4s' }} cx='780' cy='400' r='7' fill='#7DD3C0' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '3s' }} cx='1080' cy='400' r='7' fill='#E07840' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '0.9s' }} cx='220' cy='400' r='6' fill='#E07840' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '1.5s' }} cx='520' cy='400' r='6' fill='#7DD3C0' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '2.1s' }} cx='520' cy='200' r='6' fill='#E07840' />
                    <circle className='animate-node-pulse' style={{ animationDelay: '2.7s' }} cx='900' cy='200' r='6' fill='#7DD3C0' />
                </svg>
            </div>

            <div className='relative z-10 mx-auto flex min-h-[100svh] max-w-7xl flex-col justify-center px-6 pb-28 pt-28'>
                <FadeUp className='mb-8 flex items-center gap-4'>
                    <img
                        src='/logo.svg'
                        alt=''
                        width={72}
                        height={72}
                        className='size-16 rounded-[1.15rem] shadow-[0_20px_50px_-24px_rgba(224,120,64,0.55)] sm:size-[4.5rem]'
                    />
                    <div className='text-left'>
                        <p className='font-display text-5xl font-extrabold tracking-tight text-fd-foreground sm:text-6xl md:text-7xl'>
                            Ironic
                        </p>
                        <p className='mt-1 text-sm font-medium tracking-[0.18em] text-brand uppercase'>
                            Rust application framework
                        </p>
                    </div>
                </FadeUp>

                <FadeUp delay='100ms'>
                    <h1 className='max-w-3xl text-left font-display text-3xl font-bold tracking-tight text-fd-foreground sm:text-4xl md:text-5xl'>
                        Ship structured APIs without fighting the framework.
                    </h1>
                </FadeUp>

                <FadeUp delay='200ms' className='mt-5 max-w-xl'>
                    <p className='text-left text-base font-medium leading-relaxed text-fd-muted-foreground md:text-lg'>
                        Nest-style modules and DI on Axum — typed controllers, pipelines, and
                        production tooling with zero runtime reflection.
                    </p>
                </FadeUp>

                <FadeUp delay='300ms' className='mt-10 flex flex-col items-stretch gap-3 sm:flex-row sm:items-center'>
                    <Button
                        asChild
                        size='lg'
                        className='h-12 rounded-xl bg-brand px-8 text-sm font-semibold text-white shadow-[0_16px_40px_-18px_rgba(224,120,64,0.75)] transition-all hover:bg-brand/90 hover:scale-[1.02]'>
                        <Link to='/docs/getting-started'>
                            Get started
                            <ArrowRight className='ml-2 h-4 w-4' />
                        </Link>
                    </Button>
                    <Button
                        asChild
                        variant='outline'
                        size='lg'
                        className='h-12 rounded-xl border-fd-border px-8 text-sm font-semibold transition-all hover:bg-fd-accent hover:text-fd-accent-foreground'>
                        <a
                            href='https://github.com/ironic-org/ironic'
                            target='_blank'
                            rel='noopener noreferrer'
                            className='inline-flex items-center'>
                            <Github className='mr-2 h-4 w-4' />
                            View on GitHub
                            <GitHubStarButton />
                        </a>
                    </Button>
                </FadeUp>
            </div>
        </section>
    );
};

export default HeroSection;
