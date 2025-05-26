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
            // Original centering is left-1/2 and translateX(-50%)
            // We adjust translateX to shift it further left.
            // Previous was: calc(-50% - 1rem)
            // New, more to the left: calc(-50% - 2rem)
            style={{ transform: 'translateX(calc(-75% - 10rem))' }} // Shift 2rem (32px) to the left of true center
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