import { Download, Github, Tag } from 'lucide-react';
import FadeUp from './fade-up';

const stats = [
    { icon: <Download className='w-5 h-5' />, value: '0.1.3', label: 'Latest version' },
    { icon: <Github className='w-5 h-5' />, value: 'MIT', label: 'Open source license' },
    { icon: <Tag className='w-5 h-5' />, value: 'Rust 1.97', label: 'Minimum Rust version' },
];

const StatsBar = () => {
    return (
        <section className='relative py-16 px-6 max-w-7xl mx-auto border-t border-fd-border'>
            <FadeUp className='grid grid-cols-1 sm:grid-cols-3 gap-8 max-w-2xl mx-auto'>
                {stats.map((stat, i) => (
                    <div key={i} className='flex flex-col items-center text-center gap-2'>
                        <div className='text-fd-muted-foreground/60'>
                            {stat.icon}
                        </div>
                        <div className='text-2xl font-bold text-fd-foreground tracking-tight'>
                            {stat.value}
                        </div>
                        <div className='text-xs font-medium text-fd-muted-foreground uppercase tracking-wider'>
                            {stat.label}
                        </div>
                    </div>
                ))}
            </FadeUp>
        </section>
    );
};

export default StatsBar;
