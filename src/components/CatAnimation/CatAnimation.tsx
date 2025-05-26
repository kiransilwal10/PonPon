// src/components/CatAnimation/CatAnimation.tsx
import React from 'react';

import catAnimationGif from '/idle.gif'; // Assuming idle.gif is in your `public` directory

interface CatAnimationProps {
    // children?: React.ReactNode; 
}

const CatAnimation: React.FC<CatAnimationProps> = () => {
    return (
        <div
            className="absolute top-6 left-1/2 w-12 h-12 z-50 pointer-events-none"
            style={{ transform: 'translateX(calc(-75% - 7rem))' }}
        >
            <img
                src={catAnimationGif}
                alt="Animated Cat"
                className="w-full h-full object-contain"
            />
        </div>
    );
};

export default CatAnimation;