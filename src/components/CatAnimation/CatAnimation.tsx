// src/components/CatAnimation/CatAnimation.tsx
import React from 'react';

import catAnimationGif from '/idle.gif';

interface CatAnimationProps {
    children?: React.ReactNode;
}

const CatAnimation: React.FC<CatAnimationProps> = () => {
    return (
        <div className="absolute top-8 w-12 h-12 z-50">
            <img
                src={catAnimationGif}
                alt="Animated Cat"
                className="w-full h-full object-contain"
            />
        </div>


    );
};

export default CatAnimation;