import CTA from './home/components/cta';
import CodeShowcase from './home/components/code-showcase';
import Features from './home/components/features';
import Footer from './home/components/footer';
import HeroSection from './home/components/hero-section';
import Navigation from './home/components/navigation';
import StatsBar from './home/components/stats-bar';
import TechStack from './home/components/tech-stack';
import MarqueeContent from './home/components/marquee-content';

export default function HomePage() {
    return (
        <div className='relative min-h-screen'>
            <Navigation />
            <main className='relative z-10'>
                <HeroSection />


                <Features />
                <StatsBar />
                <CodeShowcase />
                <TechStack />
                <CTA />
            </main>
            <Footer />
        </div>
    );
}
