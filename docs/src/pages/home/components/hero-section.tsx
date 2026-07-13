import { Button } from '@/components/ui/button';
import { ArrowRight, Github, Terminal } from 'lucide-react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';

const HeroSection = () => {
    return (
        <section className='relative pt-28 pb-16 px-6 max-w-7xl mx-auto flex flex-col items-center text-center'>
            <FadeUp>
                <a
                    href='https://github.com/ironic-org/ironic'
                    target='_blank'
                    rel='noopener noreferrer'
                    className='group relative inline-flex items-center gap-3 rounded-full border border-fd-border bg-fd-card/50 px-4 py-1.5 text-xs font-medium text-fd-muted-foreground hover:border-fd-accent hover:text-fd-foreground transition-all mb-8'>
                    <span className='relative flex h-2 w-2'>
                        <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-brand opacity-75' />
                        <span className='relative inline-flex h-2 w-2 rounded-full bg-brand' />
                    </span>
                    v0.1.3 — now available on GitHub
                    <ArrowRight className='w-3 h-3 group-hover:translate-x-0.5 transition-transform' />
                </a>
            </FadeUp>

            <FadeUp delay='100ms'>
                <h1 className='text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-bold tracking-tighter text-fd-foreground leading-[1.05] mb-6 max-w-5xl'>
                    A type-safe
                    <br />
                    <span className='font-serif italic font-normal text-brand'>
                        application framework
                    </span>
                    <br />
                    for Rust
                </h1>
            </FadeUp>

            <FadeUp delay='200ms' className='max-w-xl mx-auto'>
                <p className='text-base md:text-lg text-fd-muted-foreground font-medium leading-relaxed mb-10'>
                    Ironic provides modules, dependency injection, controllers, pipelines,
                    lifecycle hooks, security middleware, WebSocket gateways, and an
                    Axum adapter — all without runtime reflection or global mutable state.
                </p>
            </FadeUp>

            <FadeUp
                delay='300ms'
                className='flex flex-col sm:flex-row gap-4 justify-center items-center w-full mb-16'>
                <Button
                    asChild
                    size='lg'
                    className='h-12 px-8 rounded-full bg-brand hover:bg-brand/90 text-white font-semibold text-sm shadow-lg shadow-brand/20 transition-all'>
                    <Link to='/docs'>
                        Get started
                        <ArrowRight className='ml-2 w-4 h-4' />
                    </Link>
                </Button>
                <Button
                    asChild
                    variant='outline'
                    size='lg'
                    className='h-12 px-8 rounded-full border-fd-border hover:bg-fd-accent hover:text-fd-accent-foreground font-semibold text-sm transition-all'>
                    <a href='https://github.com/ironic-org/ironic' target='_blank' rel='noopener noreferrer'>
                        <Github className='mr-2 w-4 h-4' />
                        View on GitHub
                    </a>
                </Button>
            </FadeUp>

            <FadeUp delay='400ms' className='w-full max-w-2xl'>
                <div className='relative rounded-xl border border-fd-border bg-fd-card/80 p-4 text-left shadow-sm overflow-hidden'>
                    <div className='flex items-center gap-2 mb-3 border-b border-fd-border pb-3'>
                        <Terminal className='w-4 h-4 text-fd-muted-foreground' />
                        <span className='text-xs font-medium text-fd-muted-foreground font-mono'>
                            src/users/controller.rs
                        </span>
                    </div>
                    <pre className='text-sm leading-relaxed overflow-x-auto font-mono'>
                        <code className='text-fd-foreground'>
{`#[controller("/users")]
struct UsersController;

#[routes]
impl UsersController {
    #[get("/")]
    async fn list(
        &self,
        #[query] filters: QueryFilters,
    ) -> Result<Json<Vec<User>>, HttpError> {
        Ok(Json(self.service.find_all(filters).await?))
    }
}`}</code>
                    </pre>
                </div>
            </FadeUp>
        </section>
    );
};

export default HeroSection;
