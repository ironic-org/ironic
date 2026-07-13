function StackItem({
    icon,
    name,
    desc,
}: {
    icon: React.ReactNode;
    name: string;
    desc: string;
}) {
    return (
        <div className='p-5 rounded-xl border border-fd-border bg-fd-card/50 hover:bg-fd-accent/50 hover:border-fd-accent transition-all group'>
            <div className='mb-3 text-brand group-hover:scale-110 transition-transform origin-left'>
                {icon}
            </div>
            <h4 className='text-sm font-bold text-fd-foreground mb-1'>{name}</h4>
            <p className='text-[11px] text-fd-muted-foreground font-medium uppercase tracking-wider'>
                {desc}
            </p>
        </div>
    );
}

export default StackItem;
