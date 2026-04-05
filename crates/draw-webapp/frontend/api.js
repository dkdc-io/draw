// REST API client for draw backend

const API = {
  async listDrawings() {
    const res = await fetch('/api/drawings');
    if (!res.ok) throw new Error('Failed to list drawings');
    return res.json();
  },

  async loadDrawing(id) {
    const res = await fetch(`/api/drawings/${id}`);
    if (!res.ok) throw new Error('Failed to load drawing');
    return res.json();
  },

  async saveDrawing(doc) {
    const res = await fetch('/api/drawings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    });
    if (!res.ok) throw new Error('Failed to save drawing');
    return res.json();
  },

  async deleteDrawing(id) {
    const res = await fetch(`/api/drawings/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete drawing');
  },

  async exportSvg(doc) {
    const res = await fetch('/api/export/svg', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    });
    if (!res.ok) throw new Error('Failed to export SVG');
    return res.text();
  },

  async exportPng(doc) {
    const res = await fetch('/api/export/png', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    });
    if (!res.ok) throw new Error('Failed to export PNG');
    return res.blob();
  },
};
