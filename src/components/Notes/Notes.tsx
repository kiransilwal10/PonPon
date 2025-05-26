import React, { useState } from 'react';

const Notes: React.FC = () => {
    const [notes, setNotes] = useState('');
    const [isExpanded, setIsExpanded] = useState(false);

    return (
        <div className="bg-yellow-100/80 backdrop-blur-sm rounded-lg p-3 shadow-lg">
            <div className="flex items-center justify-between mb-2">
                <h3 className="font-bold text-yellow-800">📝 Notes</h3>
                <button
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="text-yellow-600 hover:text-yellow-800"
                >
                    {isExpanded ? '▼' : '▶'}
                </button>
            </div>

            {isExpanded && (
                <textarea
                    value={notes}
                    onChange={(e) => setNotes(e.target.value)}
                    placeholder="Add your notes here..."
                    className="w-full h-20 p-2 text-sm bg-white/70 border border-yellow-300 rounded resize-none focus:outline-none focus:ring-2 focus:ring-yellow-500"
                />
            )}
        </div>
    );
};

export default Notes;
