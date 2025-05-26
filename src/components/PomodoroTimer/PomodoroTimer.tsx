import React, { useEffect, useState } from 'react';

const PomodoroTimer: React.FC = () => {
    const [minutes, setMinutes] = useState(25);
    const [seconds, setSeconds] = useState(0);
    const [isActive, setIsActive] = useState(false);
    const [mode, setMode] = useState<'work' | 'break'>('work');

    useEffect(() => {
        let interval: NodeJS.Timeout | null = null;

        if (isActive) {
            interval = setInterval(() => {
                if (seconds > 0) {
                    setSeconds(seconds - 1);
                } else if (minutes > 0) {
                    setMinutes(minutes - 1);
                    setSeconds(59);
                } else {
                    // Timer finished
                    setIsActive(false);
                    setMode(mode === 'work' ? 'break' : 'work');
                    setMinutes(mode === 'work' ? 5 : 25); // 5 min break, 25 min work
                    setSeconds(0);
                }
            }, 1000);
        } else if (!isActive && seconds !== 0 && interval) { // Ensure interval exists before clearing
            clearInterval(interval);
        }

        return () => {
            if (interval) clearInterval(interval);
        };
    }, [isActive, seconds, minutes, mode]);

    const toggleTimer = () => {
        setIsActive(!isActive);
    };

    const resetTimer = () => {
        setIsActive(false);
        setMinutes(mode === 'work' ? 25 : 5);
        setSeconds(0);
    };

    return (
        <div className="bg-white/80 backdrop-blur-sm rounded-lg p-2 shadow-md w-full h-full flex flex-col justify-center items-center">
            <div className="text-center">
                <h2 className="text-sm font-bold mb-1 text-gray-800">
                    {mode === 'work' ? '🍅 Work Time' : '☕ Break Time'}
                </h2>

                <div className="text-4xl font-mono font-bold text-red-500 mb-2">
                    {String(minutes).padStart(2, '0')}:{String(seconds).padStart(2, '0')}
                </div>

                <div className="flex justify-center space-x-2">
                    <button
                        onClick={toggleTimer}
                        className="px-3 py-1 text-xs bg-red-500 text-white rounded-md hover:bg-red-600 transition-colors"
                    >
                        {isActive ? '⏸️ Pause' : '▶️ Start'}
                    </button>

                    <button
                        onClick={resetTimer}
                        className="px-3 py-1 text-xs bg-gray-500 text-white rounded-md hover:bg-gray-600 transition-colors"
                    >
                        🔄 Reset
                    </button>
                </div>
            </div>
        </div>
    );
};

export default PomodoroTimer;