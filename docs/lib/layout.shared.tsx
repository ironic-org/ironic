import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { GitBranch } from 'lucide-react';
import { GIT_BRANCH } from './constants';

export function baseOptions(): BaseLayoutProps {
    return {
        nav: {
            title: (
                <>
                    <img
                        src='/logo.svg'
                        alt='Ironic'
                        width='24'
                        height='24'
                        className='size-5'
                    />
                    <span className='font-medium max-md:hidden'>Ironic</span>
                    <span className='inline-flex items-center gap-1 rounded-full border border-emerald-200 dark:border-emerald-800 bg-emerald-50 dark:bg-emerald-950/50 px-2 py-0.5 text-[10px] font-medium text-emerald-700 dark:text-emerald-300 ml-1.5 max-md:hidden'>
                        <span className='relative flex size-1.5'>
                            <span className='absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-75' />
                            <span className='relative inline-flex size-1.5 rounded-full bg-emerald-500' />
                        </span>
                        {GIT_BRANCH}
                    </span>
                </>
            ),
        },
    };
}
