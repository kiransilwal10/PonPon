import React from "react";
import Header from "./components/Header";
import PomodoroTimer from "./components/PomodoroTimer";
import NotesSection from "./components/NotesSection";

export default function App() {
  return (
    <div className="min-h-screen bg-mintsoft text-text font-sans p-6">
      <div className="max-w-2xl mx-auto rounded-2xl shadow-lg bg-white p-6">
        <Header />
        <PomodoroTimer />
        <NotesSection />
      </div>
    </div>
  );
}
