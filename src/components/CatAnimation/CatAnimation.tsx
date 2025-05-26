import React from 'react';

interface CatAnimationProps {
    children?: React.ReactNode;
}

const CatAnimation: React.FC<CatAnimationProps> = ({ children }) => {
    return (
        <div className="fixed inset-0 pointer-events-none z-50">
            {/* Cat animation - click-through and see-through */}
            <div className="absolute top-4 left-4 w-16 h-16 opacity-70">
                <div className="animate-bounce text-4xl">
                    🐱
                </div>
            </div>

            {/* Render children with click-through */}
            {children && (
                <div className="pointer-events-none">
                    {children}
                </div>
            )}
        </div>
    );
};

export default CatAnimation;
