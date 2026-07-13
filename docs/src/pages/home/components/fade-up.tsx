import { useEffect, useRef, useState } from 'react';

const FadeUp = ({
    children,
    delay = '0ms',
    className = '',
}: {
    children: React.ReactNode;
    delay?: string;
    className?: string;
}) => {
    const ref = useRef<HTMLDivElement>(null);
    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        const observer = new IntersectionObserver(
            entries => {
                const entry = entries[0];
                if (entry?.isIntersecting) {
                    setIsVisible(true);
                    observer.unobserve(entry.target);
                }
            },
            { threshold: 0.1 }
        );

        if (ref.current) observer.observe(ref.current);
        return () => observer.disconnect();
    }, []);

    return (
        <div
            ref={ref}
            className={`transition-all duration-1000 ease-[cubic-bezier(0.16,1,0.3,1)] ${
                isVisible
                    ? 'opacity-100 translate-y-0 blur-0'
                    : 'opacity-0 translate-y-8 blur-xs'
            } ${className}`}
            style={{ transitionDelay: delay }}>
            {children}
        </div>
    );
};

export default FadeUp;

