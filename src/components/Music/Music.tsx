import React, { useState } from 'react';

const Music: React.FC = () => {
    const [isPlaying, setIsPlaying] = useState(false);
    const [volume, setVolume] = useState(50);

    const togglePlay = () => {
        setIsPlaying(!isPlaying);
        // Here you would integrate with actual audio playback
    };

    return (
        <div className="bg-purple-100/80 backdrop-blur-sm rounded-lg p-3 shadow-lg">
            <h3 className="font-bold text-purple-800 mb-2">🎵 Music</h3>

            <div className="space-y-2">
                <div className="flex items-center space-x-2">
                    <button
                        onClick={togglePlay}
                        className="p-1 bg-purple-500 text-white rounded hover:bg-purple-600 transition-colors"
                    >
                        {isPlaying ? '⏸️' : '▶️'}
                    </button>
                    <span className="text-sm text-purple-700">
                        {isPlaying ? 'Playing...' : 'Paused'}
                    </span>
                </div>

                <div className="flex items-center space-x-2">
                    <span className="text-xs text-purple-600">🔊</span>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        value={volume}
                        onChange={(e) => setVolume(Number(e.target.value))}
                        className="flex-1 h-1 bg-purple-200 rounded-lg appearance-none cursor-pointer"
                    />
                    <span className="text-xs text-purple-600">{volume}%</span>
                </div>
            </div>
        </div>
    );
};

export default Music;
