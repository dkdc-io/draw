// Centralized theme constants — no magic strings
// Aligned with Zorto default theme (navy-blue palette)

const THEME = {
  // Canvas
  canvasBg: '#0a0f1a',
  gridColor: 'rgba(59, 130, 246, 0.08)',

  // Element defaults
  strokeColor: '#e2e8f0',
  fillColor: '#3b82f6',
  fontFamily: 'Inter, sans-serif',
  fontWeight: '500',
  fontSize: 20,

  // Selection
  accentColor: '#3b82f6',
  accentSoft: 'rgba(59, 130, 246, 0.08)',
  selectionDash: [5, 5],
  handleFill: '#ffffff',
  handleRadius: 4,
  selectionPad: 5,

  // Shapes
  cornerRadius: 12,
  hachureGap: 10,
  hachureAngle: -0.785, // -45deg
  hachureAlpha: 0.5,
  hachureLineWidth: 1.5,
  arrowheadLength: 14,
  arrowheadAngle: 0.45,

  // Quick-pick color palette (Zorto-aligned + common diagram colors)
  palette: [
    '#e2e8f0', // slate-200 (default stroke)
    '#94a3b8', // slate-400
    '#64748b', // slate-500
    '#1e293b', // slate-800
    '#0f172a', // slate-900
    '#3b82f6', // blue-500 (default fill / accent)
    '#2563eb', // blue-600
    '#1d4ed8', // blue-700
    '#60a5fa', // blue-400
    '#93c5fd', // blue-300
    '#34d399', // emerald-400 (Zorto mint)
    '#10b981', // emerald-500
    '#f59e0b', // amber-500
    '#f97316', // orange-500
    '#ef4444', // red-500
    '#ec4899', // pink-500
    '#a855f7', // purple-500
    '#8b5cf6', // violet-500
    '#ffffff', // white
    'transparent', // none (shows as X)
  ],

  // Timing
  autoSaveIntervalMs: 30_000,
};
