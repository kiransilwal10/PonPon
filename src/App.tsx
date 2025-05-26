import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';
import { useEffect } from 'react';
import CatAnimation from './components/CatAnimation/CatAnimation';
import Music from './components/Music/Music';
import Notes from './components/Notes/Notes';
import PomodoroTimer from './components/PomodoroTimer/PomodoroTimer';
import Settings from './components/Settings/Settings';

function App() {
  useEffect(() => {
    const moveToCorner = async () => {
      try {
        const appWindow = getCurrentWindow();
        // Get the current screen size using window.screen
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;

        const x = screenWidth - 400; // Use your configured width
        const y = screenHeight - 300; // Use your configured height
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
        console.error('Error positioning window:', error);
      }
    };

    moveToCorner();
  }, []);

  return (
    <>
      {/* Cat Animation Layer - Click-through and see-through */}
      <CatAnimation />

      <div className="relative z-10 h-screen flex flex-col p-4">
        {/* Top Section - Settings in top right */}
        <div className="flex justify-end mb-4">
          <Settings />
        </div>

        {/* Main Content - Timer on left, Notes/Music on right */}
        {/* Changed items-center to items-stretch */}
        <div className="flex-1 flex items-stretch space-x-4">
          {/* Left Side - Pomodoro Timer */}
          {/* This column will take up most of the width due to flex-1 */}
          <div className="flex-1 flex items-center justify-center">
            <PomodoroTimer />
          </div>
          <div className="flex flex-col w-1/3 space-y-4">
            <div className="flex-1">
              <Notes />
            </div>
            <div className="flex-1">
              <Music />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default App;
