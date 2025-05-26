import React, { useState } from 'react';

const Notes: React.FC = () => {
    const [notes, setNotes] = useState('');
    const [isExpanded, setIsExpanded] = useState(true); // Keep expanded for now to see layout

    return (
        <div className="bg-yellow-100/80 backdrop-blur-sm rounded-lg p-1.5 shadow-md flex flex-col h-full">
            <div className="flex items-center justify-between mb-1">
                <h3 className="font-semibold text-yellow-800 text-xs">📝 Notes</h3>
                <button
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="text-yellow-600 hover:text-yellow-800 text-xs p-0.5"
                >
                    {isExpanded ? '▼' : '▶'}
                </button>
            </div>

            {isExpanded && (
                <textarea
                    value={notes}
                    onChange={(e) => setNotes(e.target.value)}
                    placeholder="Add your notes here..."
                    className="w-full flex-1 p-1 text-xs bg-white/70 border border-yellow-300 rounded resize-none focus:outline-none focus:ring-1 focus:ring-yellow-500"
                />
            )}
        </div>
    );
};

export default Notes;