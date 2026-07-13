import TechLogo from './tech-logo';

const TECH_STACK = [
    {
        name: 'Rust',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
    { name: 'Axum', src: 'https://cdn.worldvectorlogo.com/logos/rust.svg' },
    {
        name: 'Tokio',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
    {
        name: 'Tower',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
    {
        name: 'Serde',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
    { name: 'Tracing', src: 'https://cdn.worldvectorlogo.com/logos/rust.svg' },
    {
        name: 'Cargo',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
    {
        name: 'Tower layers',
        src: 'https://cdn.worldvectorlogo.com/logos/rust.svg',
    },
];

function MarqueeContent() {
    return (
        <div className='flex animate-marquee whitespace-nowrap gap-16 items-center min-w-full pr-16'>
            {TECH_STACK.map((tech, i) => (
                <TechLogo key={i} name={tech.name} src={tech.src} />
            ))}
            {/* Duplicate for seamless loop */}
            {TECH_STACK.map((tech, i) => (
                <TechLogo key={`dup-${i}`} name={tech.name} src={tech.src} />
            ))}
        </div>
    );
}
export default MarqueeContent;
