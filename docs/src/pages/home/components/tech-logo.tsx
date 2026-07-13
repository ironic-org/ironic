import { Box } from 'lucide-react';

function TechLogo({ name, src }: { name: string; src?: string }) {
  return (
    <div className="group/logo flex items-center gap-2 opacity-50 transition-all duration-300 hover:opacity-100">
      <div className="relative flex h-8 w-8 items-center justify-center grayscale transition-all group-hover/logo:grayscale-0">
        {src ? (
          <img src={src} alt={name} className="h-full w-full object-contain" />
        ) : (
          <div className="flex h-8 w-8 items-center justify-center rounded bg-white/10">
            <Box className="h-4 w-4 text-gray-400" />
          </div>
        )}
      </div>
      <span className="cursor-default text-sm font-bold text-gray-500 transition-colors group-hover/logo:text-gray-200">
        {name}
      </span>
    </div>
  );
}

export default TechLogo;
