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
        <div className='p-6 rounded-2xl border border-white/3 bg-white/2 hover:bg-white/5 hover:border-white/10 transition-all group'>
            <div className='mb-4 text-[#00b7a8] group-hover:scale-110 transition-transform origin-left'>
                {icon}
            </div>
            <h4 className='text-white font-bold mb-1'>{name}</h4>
            <p className='text-[11px] text-gray-600 font-bold uppercase tracking-wider'>
                {desc}
            </p>
        </div>
    );
}

export default StackItem;
