import CTA from './home/components/cta';
import CodeShowcase from './home/components/code-showcase';
import ComparisonTable from './home/components/comparison-table';
import Features from './home/components/features';
import Footer from './home/components/footer';
import HeroSection from './home/components/hero-section';
import Navigation from './home/components/navigation';
import StatsBar from './home/components/stats-bar';
import TechStack from './home/components/tech-stack';

export default function HomePage() {
    return (
        <div className='relative min-h-screen'>
            <Navigation />
            <main className='relative z-10'>
                <HeroSection />
                <Features />
                <StatsBar />
                <ComparisonTable />
                <CodeShowcase />
                <TechStack />
                <CTA />
            </main>
            <Footer />
        </div>
    );
}
