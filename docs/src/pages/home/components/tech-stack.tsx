import {
    Box,
    Cpu,
    Github,
    Globe,
    Layers,
    Palette,
    ShieldCheck,
    Sparkles,
} from 'lucide-react';
import FadeUp from './fade-up';
import StackItem from './stack-item';

const TechStack = () => {
    return (
        <section id='stack' className='relative py-24 px-6 max-w-7xl mx-auto'>
            <FadeUp className='text-center mb-16'>
                <h2 className='text-4xl md:text-6xl font-serif text-fd-foreground mb-6 tracking-tight'>
                    Modern. Scalable.
                    <br />
                    <span className='italic text-brand'>Opinionated.</span>
                </h2>
                <p className='text-fd-muted-foreground text-lg max-w-2xl mx-auto font-medium'>
                    A small kernel keeps framework contracts independent from
                    the concrete HTTP runtime and its escape hatches.
                </p>
            </FadeUp>

            <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4'>
                <StackItem
                    icon={<Globe className='w-6 h-6 text-brand' />}
                    name='Rust 2024'
                    desc='Type-safe public contracts'
                />
                <StackItem
                    icon={<Cpu className='w-6 h-6 text-brand' />}
                    name='Dependency Injection'
                    desc='Singletons and transients'
                />
                <StackItem
                    icon={<Github className='w-6 h-6 text-brand' />}
                    name='Axum'
                    desc='Default platform adapter'
                />
                <StackItem
                    icon={<Palette className='w-6 h-6 text-brand' />}
                    name='Tower'
                    desc='Layer escape hatches'
                />
                <StackItem
                    icon={<Box className='w-6 h-6 text-brand' />}
                    name='Modules'
                    desc='Validated application graphs'
                />
                <StackItem
                    icon={<ShieldCheck className='w-6 h-6 text-brand' />}
                    name='Testing'
                    desc='Socket-free HTTP harness'
                />
                <StackItem
                    icon={<Layers className='w-6 h-6 text-brand' />}
                    name='Tracing'
                    desc='Request correlation spans'
                />
                <StackItem
                    icon={<Sparkles className='w-6 h-6 text-brand' />}
                    name='CLI'
                    desc='Deterministic generators'
                />
            </div>
        </section>
    );
};

export default TechStack;
