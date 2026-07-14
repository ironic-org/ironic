import { Box, Cog, Cpu, Globe, Layers, Server, ShieldCheck, Workflow } from 'lucide-react';

const TECH_STACK = [
    { name: 'Rust', icon: <Cpu className='w-5 h-5' /> },
    { name: 'Axum', icon: <Server className='w-5 h-5' /> },
    { name: 'Tokio', icon: <Workflow className='w-5 h-5' /> },
    { name: 'Tower', icon: <Layers className='w-5 h-5' /> },
    { name: 'Serde', icon: <Box className='w-5 h-5' /> },
    { name: 'Tracing', icon: <Globe className='w-5 h-5' /> },
    { name: 'Cargo', icon: <Cog className='w-5 h-5' /> },
    { name: 'garde', icon: <ShieldCheck className='w-5 h-5' /> },
];

function MarqueeContent() {
    const items = [...TECH_STACK, ...TECH_STACK];

    return (
        <div className='flex animate-marquee whitespace-nowrap gap-12 items-center min-w-full pr-12'>
            {items.map((tech, i) => (
                <div
                    key={`${tech.name}-${i}`}
                    className='inline-flex items-center gap-2 opacity-30 hover:opacity-80 transition-all duration-300'>
                    <div className='text-brand'>{tech.icon}</div>
                    <span className='text-sm font-semibold text-fd-foreground'>
                        {tech.name}
                    </span>
                </div>
            ))}
        </div>
    );
}

export default MarqueeContent;
export { TECH_STACK };
