import React, { useEffect, useState } from 'react';

const PlayIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z" />
    </svg>
);

const PauseIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
    </svg>
);

const ResetIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" />
    </svg>
);


// --- PomodoroTimer Component ---
const PomodoroTimer: React.FC = () => {
    const WORK_MINUTES = 25;
    const BREAK_MINUTES = 5;

    const [minutes, setMinutes] = useState(WORK_MINUTES);
    const [seconds, setSeconds] = useState(0);
    const [isActive, setIsActive] = useState(false);
    const [mode, setMode] = useState<'work' | 'break'>('work');
    const [totalDuration, setTotalDuration] = useState(WORK_MINUTES * 60);

    useEffect(() => {
        setTotalDuration((mode === 'work' ? WORK_MINUTES : BREAK_MINUTES) * 60);
    }, [mode]);


    useEffect(() => {
        let interval: NodeJS.Timeout | null = null;
        if (isActive) {
            interval = setInterval(() => {
                if (seconds > 0) setSeconds(s => s - 1);
                else if (minutes > 0) {
                    setMinutes(m => m - 1);
                    setSeconds(59);
                } else {
                    setIsActive(false);
                    const nextMode = mode === 'work' ? 'break' : 'work';
                    setMode(nextMode);
                    setMinutes(nextMode === 'work' ? WORK_MINUTES : BREAK_MINUTES);
                    setSeconds(0);
                }
            }, 1000);
        } else if (interval) {
            clearInterval(interval);
        }
        return () => { if (interval) clearInterval(interval); };
    }, [isActive, seconds, minutes, mode]);

    const toggleTimer = () => setIsActive(!isActive);
    const resetTimer = () => {
        setIsActive(false);
        setMinutes(mode === 'work' ? WORK_MINUTES : BREAK_MINUTES);
        setSeconds(0);
    };

    // SVG Circular Progress Bar Calculations
    const SVG_SIZE = 120; // The width and height of the SVG viewBox
    const STROKE_WIDTH = 8; // Make stroke slightly thinner if desired, or keep at 10
    const CENTER = SVG_SIZE / 2; // Center point for cx, cy
    const RADIUS = CENTER - STROKE_WIDTH / 2; // Radius of the circle's path
    const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

    const currentTimeInSeconds = minutes * 60 + seconds;
    // Ensure progress doesn't go below 0 or above 1 due to potential timing issues at boundaries
    const progressFraction = Math.max(0, Math.min(1, (totalDuration - currentTimeInSeconds) / totalDuration));
    const strokeDashoffset = CIRCUMFERENCE * (1 - progressFraction);

    return (
        <div className="flex flex-col justify-between items-center h-full w-full text-white py-2"> {/* Added py-2 for some vertical padding */}
            {/* Circular Progress and Time */}
            {/* Adjusted container size to tightly fit the SVG, mb-2 for space */}
            <div className="relative w-[120px] h-[120px] mb-2"> {/* Match SVG_SIZE */}
                <svg
                    height={SVG_SIZE}
                    width={SVG_SIZE}
                    viewBox={`0 0 ${SVG_SIZE} ${SVG_SIZE}`}
                    className="transform -rotate-90"
                >
                    {/* Background Circle Track */}
                    <circle
                        stroke="rgba(255, 255, 255, 0.2)"
                        fill="transparent"
                        strokeWidth={STROKE_WIDTH}
                        r={RADIUS}
                        cx={CENTER}
                        cy={CENTER}
                    />
                    {/* Progress Circle */}
                    <circle
                        stroke="rgb(255, 192, 200)"
                        fill="transparent"
                        strokeWidth={STROKE_WIDTH}
                        strokeDasharray={CIRCUMFERENCE}
                        style={{ strokeDashoffset }}
                        strokeLinecap="round"
                        r={RADIUS}
                        cx={CENTER}
                        cy={CENTER}
                    />
                </svg>
                {/* Time Display - centered using flex utilities */}
                <div className="absolute inset-0 flex items-center justify-center">
                    {/* Reduced text size slightly for better fit if needed */}
                    <span className="text-2xl font-mono font-semibold text-white">
                        {String(minutes).padStart(2, '0')}:{String(seconds).padStart(2, '0')}
                    </span>
                </div>
            </div>

            {/* Controls - Use a container that spans the width and position buttons within it */}
            {/* The parent of this div is a flex column, this will be the bottom item */}
            <div className="flex justify-between items-center w-full px-6">
                <button
                    onClick={resetTimer}
                    className="p-1 text-white/80 hover:text-white transition-colors"
                    aria-label="Reset timer"
                >
                    <ResetIcon />
                </button>
                <button
                    onClick={toggleTimer}
                    className="p-1 text-white/80 hover:text-white transition-colors"
                    aria-label={isActive ? "Pause timer" : "Start timer"}
                >
                    {isActive ? <PauseIcon /> : <PlayIcon />}
                </button>
            </div>
        </div>
    );
};

export default PomodoroTimer;