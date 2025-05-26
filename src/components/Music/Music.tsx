// src/components/Music/Music.tsx
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import React, { useEffect, useState } from 'react';

// --- SVG Icons ---
const PlayIcon = ({ size = "20" }) => ( // Slightly larger central play/pause
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className="text-gray-700 hover:text-black transition-colors">
        <path d="M8 5v14l11-7z" />
    </svg>
);

const PauseIcon = ({ size = "20" }) => (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className="text-gray-700 hover:text-black transition-colors">
        <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
    </svg>
);

const PrevIcon = ({ size = "18" }) => (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className="text-gray-600 hover:text-black transition-colors">
        <path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" />
    </svg>
);

const NextIcon = ({ size = "18" }) => (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className="text-gray-600 hover:text-black transition-colors">
        <path d="M16 18h2V6h-2zM6 18l8.5-6L6 6z" />
    </svg>
);
// --- End SVG Icons ---

interface MediaInfoPayload {
    title?: string;
    artist?: string;
    is_playing: boolean;
    album_art_url?: string | null;
    current_time_ms?: number;
    total_time_ms?: number;
}

const Music: React.FC = () => {
    const [systemTrack, setSystemTrack] = useState<string | null>("No Media Detected");
    const [systemArtist, setSystemArtist] = useState<string | null>(null);
    const [isSystemPlaying, setIsSystemPlaying] = useState(false);
    const [albumArt, setAlbumArt] = useState<string | null>(null);
    const [currentTime, setCurrentTime] = useState(0); // in seconds
    const [totalTime, setTotalTime] = useState(0);     // in seconds

    useEffect(() => {
        const unlisten = listen<MediaInfoPayload>('media-info-update', (event) => {
            setSystemTrack(event.payload.title || "Unknown Title");
            setSystemArtist(event.payload.artist || "Unknown Artist");
            setIsSystemPlaying(event.payload.is_playing);
            setAlbumArt(event.payload.album_art_url || null);

            if (event.payload.total_time_ms && event.payload.total_time_ms > 0) {
                setTotalTime(event.payload.total_time_ms / 1000);
                setCurrentTime((event.payload.current_time_ms || 0) / 1000);
            } else {
                setTotalTime(0);
                setCurrentTime(0);
            }
        });
        // invoke('get_initial_media_state').catch(console.error);
        return () => { unlisten.then(f => f()); };
    }, []);

    const handlePlayPause = () => invoke('system_media_toggle_play_pause').catch(console.error);
    const handleNextTrack = () => invoke('system_media_next_track').catch(console.error);
    const handlePreviousTrack = () => invoke('system_media_previous_track').catch(console.error);

    const progressPercent = totalTime > 0 ? (currentTime / totalTime) * 100 : 0;
    const placeholderAlbumArt = "data:image/svg+xml;charset=UTF-8,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2040%2040%22%3E%3Crect%20width%3D%2240%22%20height%3D%2240%22%20fill%3D%22%23ddd%22%3E%3C%2Frect%3E%3Ctext%20x%3D%2250%25%22%20y%3D%2250%25%22%20fill%3D%22%23777%22%20dy%3D%22.3em%22%20font-size%3D%2220%22%20text-anchor%3D%22middle%22%3E%E2%99%AB%3C%2Ftext%3E%3C%2Fsvg%3E"; // Music note placeholder

    return (
        // Removed background, shadow, rounded-lg, backdrop-blur. Uses parent's background.
        // p-1.5 provides minimal internal padding.
        // text-gray-700 as a base text color for this component.
        <div className="h-full flex flex-col p-1.5 text-gray-700 overflow-hidden">
            {/* Main content: Album art, Info, Progress, Controls */}
            <div className="flex items-center space-x-2 flex-grow min-h-0">
                {/* Album Art */}
                <div className="w-10 h-10 rounded-md overflow-hidden flex-shrink-0 border border-gray-300/50"> {/* Smaller, square art */}
                    <img
                        src={albumArt || placeholderAlbumArt}
                        alt="Album"
                        className="w-full h-full object-cover"
                        onError={(e) => (e.currentTarget.src = placeholderAlbumArt)}
                    />
                </div>

                {/* Right side: Track Info, Progress, Mini Controls */}
                <div className="flex-1 flex flex-col justify-center min-w-0 h-full space-y-1"> {/* Added space-y-1 */}
                    {/* Track Info */}
                    <div className='overflow-hidden'>
                        <p className="text-xs font-semibold truncate" title={systemTrack || ""}>
                            {systemTrack || "---"}
                        </p>
                        {systemArtist && (
                            <p className="text-[10px] text-gray-600 truncate" title={systemArtist}> {/* Smaller artist text */}
                                {systemArtist}
                            </p>
                        )}
                    </div>

                    {/* Playback Progress Bar */}
                    <div className="w-full h-1 bg-gray-300/70 rounded-full overflow-hidden">
                        <div
                            className="h-full bg-purple-500 rounded-full" // Progress bar color
                            style={{ width: `${progressPercent}%`, transition: 'width 0.2s linear' }} // Linear transition
                        />
                    </div>
                </div>
            </div>

            {/* Bottom Controls: Prev, Play/Pause, Next - Centered */}
            <div className="flex items-center justify-center space-x-4 mt-1 pt-1 flex-shrink-0"> {/* Added mt-1, pt-1, larger space */}
                <button onClick={handlePreviousTrack} className="p-1 rounded-full hover:bg-gray-200/70" title="Previous">
                    <PrevIcon />
                </button>
                <button onClick={handlePlayPause} className="p-1 rounded-full hover:bg-gray-200/70" title={isSystemPlaying ? "Pause" : "Play"}>
                    {isSystemPlaying ? <PauseIcon /> : <PlayIcon />}
                </button>
                <button onClick={handleNextTrack} className="p-1 rounded-full hover:bg-gray-200/70" title="Next">
                    <NextIcon />
                </button>
            </div>
        </div>
    );
};

export default Music;