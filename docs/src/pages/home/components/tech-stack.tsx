import {
    Box,
    Cog,
    Cpu,
    Github,
    Globe,
    Layers,
    Palette,
    ShieldCheck,
} from 'lucide-react';
import FadeUp from './fade-up';
import StackItem from './stack-item';

const items = [
    { icon: <Globe className='w-6 h-6 text-brand' />, name: 'Rust 2024', desc: 'Type-safe contracts' },
    { icon: <Cpu className='w-6 h-6 text-brand' />, name: 'Axum + Tower', desc: 'HTTP platform adapter' },
    { icon: <Box className='w-6 h-6 text-brand' />, name: 'Modules', desc: 'Validated graphs' },
    { icon: <Layers className='w-6 h-6 text-brand' />, name: 'DI Container', desc: 'Scope-aware injection' },
    { icon: <ShieldCheck className='w-6 h-6 text-brand' />, name: 'Testing', desc: 'Socket-free harness' },
    { icon: <Cog className='w-6 h-6 text-brand' />, name: 'CLI Tools', desc: 'Scaffold & generate' },
    { icon: <Palette className='w-6 h-6 text-brand' />, name: 'OpenAPI', desc: 'Route discovery' },
    { icon: <Github className='w-6 h-6 text-brand' />, name: 'MIT Licensed', desc: 'Open source' },
];

const TechStack = () => {
    return (
        <section className='relative py-24 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <FadeUp className='text-center mb-16'>
                <h2 className='text-3xl md:text-4xl font-bold text-fd-foreground mb-4 tracking-tight'>
                    Built on a{' '}
                    <span className='text-brand'>proven stack</span>
                </h2>
                <p className='text-fd-muted-foreground text-base max-w-xl mx-auto font-medium'>
                    A small kernel keeps framework contracts independent of the
                    runtime. Every feature is opt-in behind compile-time flags.
                </p>
            </FadeUp>

            <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
                {items.map((item, i) => (
                    <FadeUp key={i} delay={`${i * 50}ms`}>
                        <StackItem {...item} />
                    </FadeUp>
                ))}
            </div>
        </section>
    );
};

export default TechStack;
