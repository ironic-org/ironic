const Beam = ({
    delay,
    duration,
    color = 'var(--primary)',
}: {
    delay: string;
    duration: string;
    color?: string;
}) => (
    <div
        className='absolute -top-40 -right-px w-px h-40 bg-linear-to-b from-transparent via-(--beam-color) to-transparent animate-beam opacity-0 shadow-[0_0_15px_var(--beam-color)]'
        style={
            {
                '--beam-color': color,
                animationDelay: delay,
                animationDuration: duration,
            } as React.CSSProperties
        }
    />
);

export default Beam;

