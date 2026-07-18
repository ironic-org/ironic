import { Check } from 'lucide-react';
import FadeUp from './fade-up';

const features = [
    'Validated module graphs with cycle and visibility checks',
    'Scope-aware DI: singleton, transient, and request-scoped',
    'Middleware, guards, interceptors, pipes, and exception filters',
    'WebSocket gateways with rooms and broadcasting',
    'Cron and interval scheduling with cooperative shutdown',
    'In-process integration tests with dependency overrides',
];

const CodeShowcase = () => {
    return (
        <section className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <div className='grid grid-cols-1 lg:grid-cols-2 gap-12 items-center'>
                <FadeUp>
                    <h2 className='font-display text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-6'>
                        APIs that feel{' '}
                        <span className='text-brand'>natural</span>
                    </h2>
                    <p className='text-fd-muted-foreground text-base leading-relaxed mb-8'>
                        Ironic's procedural macros generate explicit Rust calls.
                        Every annotation compiles to a public API — no runtime
                        reflection, no hidden state, no magic.
                    </p>
                    <ul className='space-y-3'>
                        {features.map((item, i) => (
                            <li key={i} className='flex items-start gap-3 text-sm text-fd-muted-foreground'>
                                <Check className='w-5 h-5 text-brand mt-0.5 shrink-0' />
                                <span>{item}</span>
                            </li>
                        ))}
                    </ul>
                </FadeUp>

                <FadeUp delay='100ms'>
                    <div className='rounded-xl border border-fd-border bg-fd-card/80 overflow-hidden shadow-sm'>
                        <div className='flex items-center gap-1.5 px-4 py-3 border-b border-fd-border bg-fd-muted/30'>
                            {['#EF4444', '#F59E0B', '#22C55E'].map((color) => (
                                <div key={color} className='w-3 h-3 rounded-full' style={{ backgroundColor: color }} />
                            ))}
                            <span className='ml-3 text-[10px] font-medium text-fd-muted-foreground font-mono uppercase tracking-wider'>
                                Cargo.toml
                            </span>
                        </div>
                        <div className='p-4'>
                            <pre className='text-xs leading-relaxed overflow-x-auto font-mono'>
                                <code className='text-fd-foreground'>
{`[dependencies]
ironic = { features = [
    "security",       # CORS, rate limiting
    "validation",     # Parse & validate pipes
    "cache",          # CacheInterceptor
    "scheduling",     # Cron & interval tasks
    "realtime",       # WebSocket gateways
    "versioning",     # API versioning
    "serialization",  # Role-based fields
    "compression",    # gzip, brotli, zstd
] }`}
                                </code>
                            </pre>
                        </div>
                    </div>
                </FadeUp>
            </div>
        </section>
    );
};

export default CodeShowcase;
