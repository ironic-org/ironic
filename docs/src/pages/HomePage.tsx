import { useEffect, useState } from 'react';
import Beam from './home/components/beam';
import CTA from './home/components/cta';
import Features from './home/components/features';
import Footer from './home/components/footer';
import HeroSection from './home/components/hero-section';
import Navigation from './home/components/navigation';
import TechStack from './home/components/tech-stack';

export default function HomePage() {
    const [mouseGlobal, setMouseGlobal] = useState({ x: '50%', y: '50%' });

    useEffect(() => {
        const handleGlobalMouse = (e: MouseEvent) => {
            setMouseGlobal({ x: `${e.clientX}px`, y: `${e.clientY}px` });
        };
        window.addEventListener('mousemove', handleGlobalMouse);
        return () => window.removeEventListener('mousemove', handleGlobalMouse);
    }, []);

    return (
        <div className='relative min-h-screen selection:bg-brand/30 selection:text-white'>
            {/* Ambient Background */}
            <div className='fixed inset-0 pointer-events-none z-0'>
                <div className='absolute top-0 left-1/2 -translate-x-1/2 w-full max-w-4xl h-100 bg-brand/5 blur-[100px] rounded-full mix-blend-screen opacity-50' />
                <div className='absolute -top-25 left-1/2 -translate-x-1/2 w-[80%] h-75 bg-brand/10 blur-[120px] rounded-full' />
                <div
                    className='fixed inset-0 transition-opacity duration-300 z-0'
                    style={{
                        background: `radial-gradient(400px circle at ${mouseGlobal.x} ${mouseGlobal.y}, color-mix(in srgb, var(--color-brand) 8%, transparent), transparent 80%)`,
                    }}
                />
            </div>

            {/* Grid Lines & Beams */}
            <div className='fixed inset-0 pointer-events-none z-0 max-w-7xl mx-auto border-x border-white/3'>
                <div className='grid grid-cols-6 md:grid-cols-12 h-full w-full'>
                    {[...Array(11)].map((_, i) => (
                        <div
                            key={i}
                            className='border-r border-white/3 h-full hidden md:block relative overflow-hidden'>
                            {i % 2 === 0 && (
                                <Beam
                                    delay={`${i * 0.5}s`}
                                    duration={`${7 + (i % 3) * 2}s`}
                                    color={i % 4 === 0 ? 'var(--color-brand)' : 'var(--color-fd-muted)'}
                                />
                            )}
                        </div>
                    ))}
                </div>
            </div>

            {/* Navigation */}
            <Navigation />

            {/* Main Content */}
            <main className='relative z-10'>
                {/* Hero Section */}
                <HeroSection />

                {/* Features Section */}
                <Features />
                {/* Tech Stack Grid */}
                <TechStack />

                {/* Final CTA */}
                <CTA />
            </main>

            {/* Footer */}
            <Footer />
        </div>
    );
}
