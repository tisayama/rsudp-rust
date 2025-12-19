'use client';

import React, { useEffect, useState } from 'react';

const PerformanceMonitor: React.FC = () => {
  const [fps, setFps] = useState(0);

  useEffect(() => {
    let frameCount = 0;
    let lastTime = performance.now();
    let animationId: number;

    const tick = () => {
      frameCount++;
      const now = performance.now();
      if (now - lastTime >= 1000) {
        setFps(frameCount);
        frameCount = 0;
        lastTime = now;
      }
      animationId = requestAnimationFrame(tick);
    };

    animationId = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(animationId);
  }, []);

  return (
    <div className="fixed bottom-4 left-4 bg-black/80 text-white px-3 py-1.5 rounded-lg font-mono text-xs shadow-lg border border-white/10 z-50 flex items-center gap-2">
      <span className={`h-2 w-2 rounded-full ${fps > 55 ? 'bg-green-500' : 'bg-yellow-500'}`}></span>
      <span>{fps} FPS</span>
    </div>
  );
};

export default PerformanceMonitor;
