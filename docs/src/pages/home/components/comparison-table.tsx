import { Check, Minus, X } from 'lucide-react';
import FadeUp from './fade-up';

const rows = [
    { feature: 'Language', nestjs: 'TypeScript / Node.js', axum: 'Rust', actix: 'Rust', loco: 'Rust', ironic: 'Rust' },
    {
        feature: 'Architecture',
        nestjs: 'Decorator-based modules',
        axum: 'Handler functions',
        actix: 'Actor system',
        loco: 'MVC (Rails-inspired)',
        ironic: 'Module graph + DI',
    },
    {
        feature: 'Dependency Injection',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        actix: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Module system',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'DI with 3 scopes',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Attribute routing',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Guards + Interceptors',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: 'Tower layer',
        actix: 'Middleware',
        loco: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Exception filters',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Middleware pipeline',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: 'Tower layers',
        actix: 'Middleware wrap',
        loco: 'Tower layers',
        ironic: 'Middleware + Guards + Interceptors',
    },
    {
        feature: 'WebSockets built-in',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<Check className='w-4 h-4 text-green-500' />),
        actix: (<Check className='w-4 h-4 text-green-500' />),
        loco: (<Check className='w-4 h-4 text-green-500' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'OpenAPI generation',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: 'Utoipa',
        actix: 'Utoipa',
        loco: 'Utoipa',
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'CLI scaffolding',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<Check className='w-4 h-4 text-green-500' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'In-process testing',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<Check className='w-4 h-4 text-green-500' />),
        actix: (<Check className='w-4 h-4 text-green-500' />),
        loco: (<Check className='w-4 h-4 text-green-500' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Rate limiting built-in',
        nestjs: 'ThrottlerModule',
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Scheduled tasks / Cron',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<Check className='w-4 h-4 text-green-500' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Security headers built-in',
        nestjs: 'Helmet',
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Feature flags',
        nestjs: (<X className='w-4 h-4 text-red-500/50' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Cache (built-in)',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Events pub/sub + DLQ',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Compression',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: 'tower-http',
        actix: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        loco: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Metrics (Prometheus)',
        nestjs: 'prom-client',
        axum: 'tower-http',
        actix: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        loco: (<Minus className='w-4 h-4 text-fd-muted-foreground/40' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Health checks',
        nestjs: (<Check className='w-4 h-4 text-green-500' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Hot-reload config',
        nestjs: (<X className='w-4 h-4 text-red-500/50' />),
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: (<Check className='w-4 h-4 text-green-500' />),
    },
    {
        feature: 'Lifecycle hooks',
        nestjs: '4 hooks',
        axum: (<X className='w-4 h-4 text-red-500/50' />),
        actix: (<X className='w-4 h-4 text-red-500/50' />),
        loco: (<X className='w-4 h-4 text-red-500/50' />),
        ironic: '15 hooks',
    },
    {
        feature: 'Learning curve',
        nestjs: 'Moderate',
        axum: 'Low',
        actix: 'Medium',
        loco: 'Low',
        ironic: 'Moderate',
    },
    {
        feature: 'Ecosystem maturity',
        nestjs: 'Mature (2017)',
        axum: 'Growing (2021)',
        actix: 'Mature (2017)',
        loco: 'Growing (2023)',
        ironic: 'Early (2026)',
    },
];

const columns = [
    { key: 'feature' as const, label: '', className: 'text-left font-semibold text-fd-foreground min-w-[160px]' },
    { key: 'nestjs' as const, label: 'NestJS', className: 'text-center font-semibold text-fd-foreground' },
    { key: 'axum' as const, label: 'Axum', className: 'text-center font-semibold text-fd-foreground' },
    { key: 'actix' as const, label: 'Actix Web', className: 'text-center font-semibold text-fd-foreground' },
    { key: 'loco' as const, label: 'Loco', className: 'text-center font-semibold text-fd-foreground' },
    { key: 'ironic' as const, label: 'Ironic', className: 'text-center font-semibold text-brand' },
];

const ComparisonTable = () => {
    return (
        <section className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <div className='absolute inset-0 pointer-events-none overflow-hidden'>
                <div className='absolute top-1/3 right-0 w-150 h-150 bg-brand/2 rounded-full blur-[120px]' />
            </div>

            <FadeUp className='text-center max-w-2xl mx-auto mb-16 relative'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground tracking-tight mb-4'>
                    How <span className='text-brand'>Ironic</span> compares
                </h2>
                <p className='text-fd-muted-foreground text-base font-medium leading-relaxed'>
                    Ironic combines NestJS's batteries-included philosophy with Rust's
                    performance and safety — zero runtime overhead on unused features.
                </p>
            </FadeUp>

            <FadeUp className='relative overflow-x-auto rounded-2xl border border-fd-border bg-fd-card/30'>
                <table className='w-full text-sm'>
                    <thead>
                        <tr className='border-b border-fd-border bg-fd-muted/30'>
                            {columns.map((col) => (
                                <th
                                    key={col.key}
                                    className={`px-3 py-3.5 text-sm whitespace-nowrap ${col.className}`}
                                >
                                    {col.label}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {rows.map((row, i) => (
                            <tr
                                key={row.feature}
                                className={`border-b border-fd-border/50 transition-colors hover:bg-fd-muted/20 ${i === rows.length - 1 ? 'border-b-0' : ''
                                    }`}
                            >
                                {columns.map((col) => (
                                    <td
                                        key={col.key}
                                        className={`px-3 py-3 text-fd-muted-foreground leading-relaxed ${col.key === 'feature'
                                            ? 'font-medium text-fd-foreground text-xs tracking-wide'
                                            : col.key === 'ironic'
                                                ? 'text-center text-xs bg-brand/4 font-semibold text-brand'
                                                : 'text-center text-xs'
                                            }`}
                                    >
                                        {row[col.key]}
                                    </td>
                                ))}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </FadeUp>

            <FadeUp className='text-center mt-6'>
                <p className='text-xs text-fd-muted-foreground/60'>
                    Checkmark = built-in &nbsp;|&nbsp; Dash = third-party &nbsp;|&nbsp; Cross = not available
                </p>
            </FadeUp>
        </section>
    );
};

export default ComparisonTable;
