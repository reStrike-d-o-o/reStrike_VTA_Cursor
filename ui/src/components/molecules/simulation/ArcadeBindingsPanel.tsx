import React from 'react';

type Binding = { action: string; default: string };

const defaultBindings: Binding[] = [
  { action: 'Blue Punch (1pt)', default: 'J' },
  { action: 'Blue Body (2pt)', default: 'K' },
  { action: 'Blue Head (3pt)', default: 'L' },
  { action: 'Blue Tech Body (4pt)', default: 'U' },
  { action: 'Blue Tech Head (5pt)', default: 'I' },
  { action: 'Red Punch (1pt)', default: '1 / Num1' },
  { action: 'Red Body (2pt)', default: '2 / Num2' },
  { action: 'Red Head (3pt)', default: '3 / Num3' },
  { action: 'Red Tech Body (4pt)', default: '4 / Num4' },
  { action: 'Red Tech Head (5pt)', default: '5 / Num5' },
];

const ArcadeBindingsPanel: React.FC = () => {
  return (
    <div className="space-y-3">
      <div className="text-sm text-gray-300">Controller mapping (preview). Keyboard defaults shown; Gamepad mapping coming next.</div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
        {defaultBindings.map((b) => (
          <div key={b.action} className="flex items-center justify-between p-2 border border-gray-700">
            <span className="text-sm text-gray-200">{b.action}</span>
            <span className="text-xs text-gray-400">{b.default}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ArcadeBindingsPanel;


