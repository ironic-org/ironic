import {
    Box,
    Clock,
    Globe,
    Layers,
    ShieldCheck,
    Webhook,
} from 'lucide-react';
import FadeUp from './fade-up';

const features = [
    {
        icon: <Box className='w-6 h-6 text-brand' />,
        title: 'Modular Architecture',
        desc: 'Compose services and controllers through validated module graphs with strongly typed providers.',
        tag: 'Core',
    },
    {
        icon: <Layers className='w-6 h-6 text-brand' />,
        title: 'Dependency Injection',
        desc: 'Scope-aware DI with singletons, transients, and request-scoped providers. Optional and eager resolution.',
        tag: 'Core',
    },
    {
        icon: <ShieldCheck className='w-6 h-6 text-brand' />,
        title: 'Security Middleware',
        desc: 'CORS, rate limiting, CSRF protection, and security headers. Feature-flagged, zero overhead when disabled.',
        tag: 'Production',
    },
    {
        icon: <Clock className='w-6 h-6 text-brand' />,
        title: 'Task Scheduling',
        desc: 'Fixed-interval and cron-based scheduling with cooperative shutdown. Lifecycle-integrated background jobs.',
        tag: 'Services',
    },
    {
        icon: <Webhook className='w-6 h-6 text-brand' />,
        title: 'WebSocket Gateways',
        desc: 'Real-time bidirectional communication with event-driven message routing, rooms, and broadcasting.',
        tag: 'Realtime',
    },
    {
        icon: <Globe className='w-6 h-6 text-brand' />,
        title: 'API Versioning & Pipes',
        desc: 'URI, header, and media-type versioning. Parse and validate request parameters with composable pipes.',
        tag: 'HTTP',
    },
];

const Features = () => {
    return (
        <section
            id='features'
            className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <FadeUp className='text-center max-w-2xl mx-auto mb-16'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-4'>
                    Everything you need to build{' '}
                    <span className='text-brand'>production APIs</span>
                </h2>
                <p className='text-fd-muted-foreground text-base font-medium leading-relaxed'>
                    From controllers and DI to WebSocket gateways and cron scheduling
                    — everything is modular, type-safe, and feature-flagged.
                </p>
            </FadeUp>

            <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6'>
                {features.map((feature, i) => (
                    <FadeUp key={i} delay={`${i * 100}ms`}>
                        <div className='group relative rounded-xl border border-fd-border bg-fd-card p-6 hover:border-fd-accent hover:shadow-sm transition-all'>
                            <div className='mb-4 inline-flex h-12 w-12 items-center justify-center rounded-lg border border-fd-border bg-fd-muted/50 group-hover:bg-brand/10 group-hover:border-brand/20 transition-colors'>
                                {feature.icon}
                            </div>
                            <div className='flex items-center gap-2 mb-2'>
                                <span className='text-[10px] font-bold text-fd-muted-foreground bg-fd-muted px-2 py-0.5 rounded border border-fd-border uppercase tracking-wider'>
                                    {feature.tag}
                                </span>
                            </div>
                            <h3 className='text-lg font-bold text-fd-foreground mb-2 tracking-tight'>
                                {feature.title}
                            </h3>
                            <p className='text-sm text-fd-muted-foreground leading-relaxed'>
                                {feature.desc}
                            </p>
                        </div>
                    </FadeUp>
                ))}
            </div>
        </section>
    );
};

export default Features;
