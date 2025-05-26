import React, { useState } from 'react';

const Settings: React.FC = () => {
    const [isOpen, setIsOpen] = useState(false);
    const [workDuration, setWorkDuration] = useState(25);
    const [breakDuration, setBreakDuration] = useState(5);

    return (
        <div className="relative">
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="p-2 bg-gray-600 text-white rounded-full hover:bg-gray-700 transition-colors"
                title="Settings"
            >
                ⚙️
            </button>

            {isOpen && (
                <div className="absolute bottom-full right-0 mb-2 bg-white/90 backdrop-blur-sm rounded-lg p-4 shadow-lg min-w-[200px]">
                    <h3 className="font-bold mb-3 text-gray-800">Settings</h3>

                    <div className="space-y-3">
                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                Work Duration (minutes)
                            </label>
                            <input
                                type="number"
                                value={workDuration}
                                onChange={(e) => setWorkDuration(Number(e.target.value))}
                                className="w-full px-2 py-1 border border-gray-300 rounded text-sm"
                                min="1"
                                max="60"
                            />
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                Break Duration (minutes)
                            </label>
                            <input
                                type="number"
                                value={breakDuration}
                                onChange={(e) => setBreakDuration(Number(e.target.value))}
                                className="w-full px-2 py-1 border border-gray-300 rounded text-sm"
                                min="1"
                                max="30"
                            />
                        </div>

                        <button
                            onClick={() => setIsOpen(false)}
                            className="w-full px-3 py-1 bg-blue-500 text-white rounded text-sm hover:bg-blue-600 transition-colors"
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
