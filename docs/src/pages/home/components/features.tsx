import { Box, Clock, Globe, Layers, ShieldCheck, Webhook } from 'lucide-react';
import FadeUp from './fade-up';
import SpotlightCard from './spot-light-card';

const features = [
    {
        icon: <Box className='w-6 h-6' />,
        title: 'Modular Architecture',
        desc: 'Compose services and controllers through validated module graphs with strongly typed providers.',
        tag: 'Core',
    },
    {
        icon: <Layers className='w-6 h-6' />,
        title: 'Dependency Injection',
        desc: 'Scope-aware DI with singletons, transients, and request-scoped providers. Optional and eager resolution.',
        tag: 'Core',
    },
    {
        icon: <ShieldCheck className='w-6 h-6' />,
        title: 'Security Middleware',
        desc: 'CORS, rate limiting, CSRF protection, and security headers — feature-flagged, zero overhead when disabled.',
        tag: 'Production',
    },
    {
        icon: <Clock className='w-6 h-6' />,
        title: 'Task Scheduling',
        desc: 'Fixed-interval and cron-based scheduling with cooperative shutdown. Lifecycle-integrated background jobs.',
        tag: 'Services',
    },
    {
        icon: <Webhook className='w-6 h-6' />,
        title: 'WebSocket Gateways',
        desc: 'Real-time bidirectional communication with event-driven message routing, rooms, and broadcasting.',
        tag: 'Realtime',
    },
    {
        icon: <Globe className='w-6 h-6' />,
        title: 'API Versioning & Pipes',
        desc: 'URI, header, and media-type versioning. Parse and validate request parameters with composable pipes.',
        tag: 'HTTP',
    },
];

const Features = () => {
    return (
        <section id='features' className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <div className='absolute inset-0 pointer-events-none overflow-hidden'>
                <div className='absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-brand/[0.03] rounded-full blur-[150px]' />
            </div>

            <FadeUp className='text-center max-w-2xl mx-auto mb-16 relative'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-4'>
                    Everything you need to build{' '}
                    <span className='text-brand'>production APIs</span>
                </h2>
                <p className='text-fd-muted-foreground text-base font-medium leading-relaxed'>
                    From controllers and DI to WebSocket gateways and cron scheduling
                    — modular, type-safe, and feature-flagged.
                </p>
            </FadeUp>

            <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative'>
                {features.map((feature, i) => (
                    <FadeUp key={feature.title} delay={`${i * 100}ms`}>
                        <SpotlightCard
                            delay={`${i * 100}ms`}
                            className='h-full'>
                            <div className='mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl border border-fd-border bg-brand/10 text-brand group-hover:bg-brand/15 transition-colors'>
                                {feature.icon}
                            </div>
                            <div className='flex items-center gap-2 mb-3'>
                                <span className='text-[10px] font-bold text-fd-muted-foreground bg-fd-muted px-2 py-0.5 rounded-full border border-fd-border uppercase tracking-wider'>
                                    {feature.tag}
                                </span>
                            </div>
                            <h3 className='text-lg font-bold text-fd-foreground mb-2 tracking-tight'>
                                {feature.title}
                            </h3>
                            <p className='text-sm text-fd-muted-foreground leading-relaxed'>
                                {feature.desc}
                            </p>
                        </SpotlightCard>
                    </FadeUp>
                ))}
            </div>
        </section>
    );
};

export default Features;
