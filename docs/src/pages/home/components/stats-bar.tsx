import { Download, Github, Sparkles } from 'lucide-react';
import FadeUp from './fade-up';

const stats = [
    { icon: <Download className='w-5 h-5' />, value: '0.4.6', label: 'Latest version' },
    { icon: <Github className='w-5 h-5' />, value: 'MIT', label: 'Open source license' },
    { icon: <Sparkles className='w-5 h-5' />, value: 'Rust 1.97+', label: 'Minimum Rust version' },
];

const StatsBar = () => {
    return (
        <section className='relative py-16 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <div className='absolute inset-0 bg-brand/2 -mx-6' />
            <FadeUp className='grid grid-cols-1 sm:grid-cols-3 gap-8 max-w-2xl mx-auto relative'>
                {stats.map((stat) => (
                    <div key={stat.label} className='flex flex-col items-center text-center gap-3 p-6 rounded-2xl hover:bg-fd-card/50 transition-colors'>
                        <div className='flex items-center justify-center w-12 h-12 rounded-xl bg-brand/10 text-brand'>
                            {stat.icon}
                        </div>
                        <div>
                            <div className='text-2xl font-bold text-fd-foreground tracking-tight'>
                                {stat.value}
                            </div>
                            <div className='text-xs font-medium text-fd-muted-foreground uppercase tracking-wider mt-1'>
                                {stat.label}
                            </div>
                        </div>
                    </div>
                ))}
            </FadeUp>
        </section>
    );
};

export default StatsBar;
