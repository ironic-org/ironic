import { Button } from '@/components/ui/button';
import { ArrowRight, Copy, Github, Terminal } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import FadeUp from './fade-up';

const HeroSection = () => {
    const [copied, setCopied] = useState(false);
    const installCmd = 'cargo install ironic';

    function handleCopy() {
        navigator.clipboard.writeText(installCmd);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    }

    return (
        <section className='relative pt-28 pb-16 px-6 max-w-7xl mx-auto flex flex-col items-center text-center overflow-hidden'>
            <div className='absolute inset-0 pointer-events-none overflow-hidden'>
                <div className='absolute -top-40 left-1/4 w-150 h-150 bg-brand/8 rounded-full blur-[120px] animate-glow' />
                <div className='absolute -bottom-40 right-1/4 w-125 h-125 bg-brand/5 rounded-full blur-[100px] animate-glow' style={{ animationDelay: '1s' }} />
            </div>

            <FadeUp>
                <a
                    href='https://github.com/ironic-org/ironic'
                    target='_blank'
                    rel='noopener noreferrer'
                    className='group relative inline-flex items-center gap-3 rounded-full border border-fd-border bg-fd-card/50 px-4 py-1.5 text-xs font-medium text-fd-muted-foreground hover:border-fd-accent hover:text-fd-foreground transition-all mb-8'>
                    <span className='relative flex h-2 w-2'>
                        <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75' />
                        <span className='relative inline-flex h-2 w-2 rounded-full bg-emerald-500' />
                    </span>
                    v0.2.8 — available on GitHub
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
                    Modules, DI, controllers, pipelines, lifecycle hooks, WebSocket
                    gateways, and an Axum adapter — zero runtime reflection, no global
                    mutable state.
                </p>
            </FadeUp>

            <FadeUp delay='300ms' className='flex flex-col sm:flex-row gap-4 justify-center items-center w-full mb-8'>
                <Button
                    asChild
                    size='lg'
                    className='h-12 px-8 rounded-full bg-brand hover:bg-brand/90 text-white font-semibold text-sm shadow-lg shadow-brand/20 transition-all hover:shadow-brand/30 hover:scale-[1.02]'>
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

            <FadeUp delay='350ms' className='mb-6'>
                <button
                    onClick={handleCopy}
                    className='inline-flex items-center gap-2 rounded-full border border-fd-border bg-fd-card/30 px-5 py-2.5 text-xs font-mono text-fd-muted-foreground hover:border-brand/40 hover:text-fd-foreground hover:bg-brand/5 transition-all'>
                    <Terminal className='w-3.5 h-3.5 text-brand' />
                    <span>{installCmd}</span>
                    <span className='text-[10px] text-fd-muted-foreground/60 ml-1 hidden sm:inline'>
                        -- one command
                    </span>
                    <Copy className='w-3 h-3 ml-1 opacity-50' />
                    {copied && (
                        <span className='text-[10px] text-emerald-500 font-medium animate-in'>
                            Copied!
                        </span>
                    )}
                </button>
            </FadeUp>

            <FadeUp delay='400ms' className='w-full max-w-2xl relative'>
                <div className='absolute inset-0 bg-brand/5 rounded-xl blur-xl' />
                <div className='relative rounded-xl border border-fd-border bg-fd-card/90 p-4 text-left shadow-lg'>
                    <div className='flex items-center gap-1.5 mb-3 border-b border-fd-border pb-3'>
                        <span className='w-2.5 h-2.5 rounded-full bg-red-400' />
                        <span className='w-2.5 h-2.5 rounded-full bg-amber-400' />
                        <span className='w-2.5 h-2.5 rounded-full bg-emerald-400' />
                        <span className='ml-3 text-[11px] font-medium text-fd-muted-foreground font-mono'>
                            src/controller.rs
                        </span>
                    </div>
                    <pre className='text-sm leading-relaxed overflow-x-auto font-mono'>
                        <code className='text-fd-foreground'>
                            {`#[controller("/users")]
struct UsersController;

#[routes]
impl UsersController {
    #[get("/:id")]
    async fn get(
        &self,
        #[param] id: u64,
    ) -> Result<Json<UserView>, HttpError> {
        Ok(Json(self.service.find(id).await?))
    }
}`}</code>
                    </pre>
                </div>
            </FadeUp>
        </section>
    );
};

export default HeroSection;
