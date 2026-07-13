import { Badge } from '@/components/ui/badge';
import { GitMerge, ShieldCheck } from 'lucide-react';
import FadeUp from './fade-up';
import SpotlightCard from './spot-light-card';

const Features = () => {
    return (
        <section
            id='features'
            className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <FadeUp className='text-center max-w-2xl mx-auto mb-16'>
                <h2 className='text-4xl md:text-5xl font-serif italic text-fd-foreground tracking-tight mb-4'>
                    Build Rust APIs{' '}
                    <span className='text-brand not-italic font-sans font-bold'>
                        easy to operate.
                    </span>
                </h2>
                <p className='text-fd-muted-foreground text-lg font-medium leading-relaxed'>
                    Modular dependency injection, transport-neutral HTTP
                    contracts, lifecycle hooks, and an Axum runtime.
                </p>
            </FadeUp>

            <div className='grid grid-cols-1 md:grid-cols-3 gap-6'>
                <SpotlightCard delay='0ms'>
                    <div className='h-40 w-full mb-6 rounded-xl bg-fd-card border border-fd-border relative overflow-hidden flex flex-col items-center justify-center p-4 select-none'>
                        <div className='w-20 h-20 rounded-2xl bg-linear-to-tr from-brand/20 to-fd-muted border border-fd-border flex items-center justify-center shadow-2xl'>
                            <GitMerge className='w-10 h-10 text-brand' />
                        </div>
                        <div className='absolute bottom-2 left-1/2 -translate-x-1/2 w-1/2 h-1 bg-fd-muted rounded-full overflow-hidden'>
                            <div className='h-full bg-brand w-2/3 animate-pulse' />
                        </div>
                    </div>
                    <div className='flex items-center gap-2 mb-2'>
                        <span className='text-[10px] font-bold text-brand bg-brand/10 px-2 py-0.5 rounded border border-brand/20'>
                            API
                        </span>
                    </div>
                    <h3 className='text-xl text-fd-foreground font-bold mb-2 tracking-tight'>
                        Modular Architecture
                    </h3>
                    <p className='text-sm text-fd-muted-foreground leading-relaxed font-medium'>
                        Compose services and controllers through validated
                        module graphs and strongly typed providers.
                    </p>
                </SpotlightCard>

                <SpotlightCard delay='100ms'>
                    <div className='h-40 w-full mb-6 rounded-xl bg-fd-card border border-fd-border relative overflow-hidden flex items-center justify-center select-none'>
                        <div className='text-center'>
                            <div className='text-5xl font-bold text-brand tracking-tighter'>
                                780ns
                            </div>
                            <div className='text-[10px] text-fd-muted-foreground uppercase tracking-widest mt-2 font-bold'>
                                Request Overhead
                            </div>
                        </div>
                        <div className='absolute bottom-0 left-0 w-full h-0.5 bg-fd-muted'>
                            <div
                                className='h-full bg-brand w-1/2 animate-beam'
                                style={
                                    {
                                        animationDuration: '2s',
                                    } as React.CSSProperties
                                }
                            />
                        </div>
                    </div>
                    <div className='flex items-center gap-2 mb-2'>
                        <span className='text-[10px] font-bold text-brand bg-brand/10 px-2 py-0.5 rounded border border-brand/20'>
                            TESTING
                        </span>
                    </div>
                    <h3 className='text-xl text-fd-foreground font-bold mb-2 tracking-tight'>
                        In-process Tests
                    </h3>
                    <p className='text-sm text-fd-muted-foreground leading-relaxed font-medium'>
                        Exercise complete request pipelines with dependency
                        overrides and no network socket or global state.
                    </p>
                </SpotlightCard>

                <SpotlightCard delay='200ms'>
                    <div className='h-40 w-full mb-6 rounded-xl bg-fd-card border border-fd-border relative overflow-hidden flex items-center justify-center select-none'>
                        <div className='w-16 h-16 rounded-full border-2 border-dashed border-brand/50 flex items-center justify-center relative animate-spin-slow'>
                            <ShieldCheck className='w-8 h-8 text-brand' />
                        </div>
                        <div className='absolute top-4 right-4'>
                            <Badge
                                variant='outline'
                                className='bg-brand/5 text-brand border-brand/20'>
                                SECURE
                            </Badge>
                        </div>
                    </div>
                    <div className='flex items-center gap-2 mb-2'>
                        <span className='text-[10px] font-bold text-brand bg-brand/10 px-2 py-0.5 rounded border border-brand/20'>
                            SECURE
                        </span>
                    </div>
                    <h3 className='text-xl text-fd-foreground font-bold mb-2 tracking-tight'>
                        Safe Defaults
                    </h3>
                    <p className='text-sm text-fd-muted-foreground leading-relaxed font-medium'>
                        Request IDs, structured tracing, body limits, timeouts,
                        health checks, and redacted configuration secrets.
                    </p>
                </SpotlightCard>
            </div>
        </section>
    );
};

export default Features;
