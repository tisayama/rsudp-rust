'use client';

import React from 'react';

interface IntensityBadgeProps {
  maxClass: string;
}

const JMA_COLORS: Record<string, string> = {
  '0': '#F2F2FF',
  '1': '#F2F2FF',
  '2': '#00AAFF',
  '3': '#0041FF',
  '4': '#FAE696',
  '5-': '#FFE600',
  '5+': '#FF9900',
  '6-': '#FF2800',
  '6+': '#A50021',
  '7': '#B40068',
};

function getJMAColor(shindoClass: string): string {
  return JMA_COLORS[shindoClass] || '#F2F2FF';
}

function needsDarkText(shindoClass: string): boolean {
  return ['0', '1', '4', '5-'].includes(shindoClass);
}

const IntensityBadge: React.FC<IntensityBadgeProps> = ({ maxClass }) => {
  const bgColor = getJMAColor(maxClass);
  const textColor = needsDarkText(maxClass) ? '#333333' : '#FFFFFF';

  return (
    <div
      className="fixed top-4 right-4 z-50 rounded-lg px-4 py-2 shadow-lg flex flex-col items-center min-w-[60px]"
      style={{ backgroundColor: bgColor }}
    >
      <span
        className="text-3xl font-bold leading-tight"
        style={{ color: textColor }}
      >
        {maxClass}
      </span>
      <span
        className="text-xs opacity-80"
        style={{ color: textColor }}
      >
        Intensity
      </span>
    </div>
  );
};

export default IntensityBadge;
