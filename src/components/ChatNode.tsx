import React, { ReactNode } from 'react';

export interface ChatNodeProps {
    title?: ReactNode;
    children: ReactNode;
    headerClassName?: string;
    containerClassName?: string;
    handles?: ReactNode;
    hideHeader?: boolean;
}

export const ChatNode = ({
    title,
    children,
    headerClassName = "border-b border-border-light dark:border-border-dark",
    containerClassName = "",
    handles,
    hideHeader = false
}: ChatNodeProps) => {
    return (
        <div className={`absolute bg-surface-light dark:bg-surface-dark rounded-xl shadow-lg border border-border-light dark:border-border-dark z-10 flex flex-col ${containerClassName}`}>
            {!hideHeader && (
                <div className={`p-3 flex items-center justify-between ${headerClassName}`}>
                    <div className="flex items-center gap-2">
                        {typeof title === 'string' ? <span className="font-medium text-sm text-slate-700 dark:text-slate-200">{title}</span> : title}
                    </div>
                    <span className="material-icons-round text-slate-400 text-sm">more_horiz</span>
                </div>
            )}

            {children}

            {handles}
        </div>
    );
};

export const Handle = ({ position }: { position: 'top' | 'right' | 'bottom' | 'left' }) => {
    let posClass = "";
    switch (position) {
        case 'right': posClass = "top-1/2 -right-1.5 -translate-y-1/2"; break;
        case 'left': posClass = "top-1/2 -left-1.5 -translate-y-1/2"; break;
        case 'bottom': posClass = "-bottom-1.5 left-1/2 -translate-x-1/2"; break;
        case 'top': posClass = "-top-1.5 left-1/2 -translate-x-1/2"; break;
    }

    return (
        <div className={`absolute w-3 h-3 bg-slate-400 dark:bg-slate-500 rounded-full border-2 border-white dark:border-slate-800 cursor-pointer hover:scale-125 transition-transform ${posClass}`}></div>
    );
};
