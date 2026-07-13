import { useRef, useState } from 'react';

const SpotlightCard = ({
    children,
    className,
    delay = '0ms',
    beamColor,
}: {
    children: React.ReactNode;
    className?: string;
    delay?: string;
    beamColor?: string;
}) => {
    const cardRef = useRef<HTMLDivElement>(null);
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 });

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!cardRef.current) return;
        const rect = cardRef.current.getBoundingClientRect();
        setMousePos({
            x: e.clientX - rect.left,
            y: e.clientY - rect.top,
        });
    };

    return (
        <div
            ref={cardRef}
            onMouseMove={handleMouseMove}
            className={`group relative rounded-2xl bg-fd-card/20 border border-fd-border p-6 overflow-hidden hover:bg-fd-accent/20 transition-all duration-500 fade-up-element ${className}`}
            style={{ transitionDelay: delay }}>
            <div
                className='absolute inset-0 pointer-events-none opacity-0 group-hover:opacity-10 transition-opacity duration-500'
                style={{
                    background: `radial-gradient(600px circle at ${
                        mousePos.x
                    }px ${mousePos.y}px, ${
                        beamColor || 'var(--color-brand)'
                    }, transparent 40%)`,
                }}
            />
            {children}
        </div>
    );
};

export default SpotlightCard;
