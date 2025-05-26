import React, { useState } from 'react';

const Settings: React.FC = () => {
    const [isOpen, setIsOpen] = useState(false);
    const [workDuration, setWorkDuration] = useState(25);
    const [breakDuration, setBreakDuration] = useState(5);

    return (
        <div className="relative">
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="p-1 bg-gray-600 text-white rounded-full hover:bg-gray-700 transition-colors text-xs"
                title="Settings"
            >
                ⚙️
            </button>

            {isOpen && (
                <div className="absolute bottom-full right-0 mb-1 bg-white/90 backdrop-blur-sm rounded-md p-2 shadow-md min-w-[160px]">
                    <h3 className="font-semibold mb-1.5 text-gray-800 text-sm">Settings</h3>

                    <div className="space-y-1.5">
                        <div>
                            <label className="block text-xs font-medium text-gray-700 mb-0.5">
                                Work Duration (minutes)
                            </label>
                            <input
                                type="number"
                                value={workDuration}
                                onChange={(e) => setWorkDuration(Number(e.target.value))}
                                className="w-full px-1.5 py-0.5 border border-gray-300 rounded text-xs"
                                min="1"
                                max="60"
                            />
                        </div>

                        <div>
                            <label className="block text-xs font-medium text-gray-700 mb-0.5">
                                Break Duration (minutes)
                            </label>
                            <input
                                type="number"
                                value={breakDuration}
                                onChange={(e) => setBreakDuration(Number(e.target.value))}
                                className="w-full px-1.5 py-0.5 border border-gray-300 rounded text-xs"
                                min="1"
                                max="30"
                            />
                        </div>

                        <button
                            onClick={() => setIsOpen(false)} // Consider actually saving settings here
                            className="w-full px-2 py-0.5 bg-blue-500 text-white rounded text-xs hover:bg-blue-600 transition-colors"
                        >
                            Save
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
};

export default Settings;