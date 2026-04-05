// Canvas rendering engine — thin blit layer over WASM DrawEngine

class DrawCanvas {
  constructor(canvasEl, engine) {
    this.canvas = canvasEl;
    this.ctx = canvasEl.getContext('2d');
    this.engine = engine;
    this.dirty = true;
    this.gridSize = 20;

    this.resize();
    window.addEventListener('resize', () => this.resize());
    this.animate();
  }

  resize() {
    const dpr = window.devicePixelRatio || 1;
    const rect = this.canvas.getBoundingClientRect();
    this.canvas.width = rect.width * dpr;
    this.canvas.height = rect.height * dpr;
    this.width = rect.width;
    this.height = rect.height;
    this.engine.set_size(Math.round(rect.width), Math.round(rect.height));
    this.dirty = true;
  }

  screenToWorld(sx, sy) {
    const json = this.engine.screen_to_world(sx, sy);
    return JSON.parse(json);
  }

  worldToScreen(wx, wy) {
    const zoom = this.engine.zoom();
    const scrollX = this.engine.scroll_x();
    const scrollY = this.engine.scroll_y();
    return {
      x: wx * zoom + scrollX,
      y: wy * zoom + scrollY,
    };
  }

  setZoom(newZoom, cx, cy) {
    const oldZoom = this.engine.zoom();
    const clamped = Math.max(0.1, Math.min(10, newZoom));
    const scrollX = this.engine.scroll_x();
    const scrollY = this.engine.scroll_y();
    const newScrollX = cx - (cx - scrollX) * (clamped / oldZoom);
    const newScrollY = cy - (cy - scrollY) * (clamped / oldZoom);
    this.engine.set_viewport(newScrollX, newScrollY, clamped);
    this.dirty = true;
  }

  pan(dx, dy) {
    const scrollX = this.engine.scroll_x();
    const scrollY = this.engine.scroll_y();
    const zoom = this.engine.zoom();
    this.engine.set_viewport(scrollX + dx, scrollY + dy, zoom);
    this.dirty = true;
  }

  markDirty() {
    this.dirty = true;
  }

  animate() {
    if (this.dirty) {
      this.render();
      this.dirty = false;
    }
    requestAnimationFrame(() => this.animate());
  }

  render() {
    const data = this.engine.render();
    const rw = this.engine.render_width();
    const rh = this.engine.render_height();
    if (rw === 0 || rh === 0) return;

    // Resize canvas backing store if needed
    if (this.canvas.width !== rw || this.canvas.height !== rh) {
      this.canvas.width = rw;
      this.canvas.height = rh;
    }

    // Blit WASM-rendered pixels (shapes, fills, grid, selection)
    const imgData = new ImageData(new Uint8ClampedArray(data), rw, rh);
    this.ctx.putImageData(imgData, 0, 0);

    // Overlay text using browser-native text rendering
    // (tiny-skia renders text as placeholder rects; real text uses ctx.fillText)
    this.renderTextOverlays();
  }

  renderTextOverlays() {
    const json = this.engine.get_text_overlays();
    let overlays;
    try { overlays = JSON.parse(json); } catch { return; }
    if (!overlays.length) return;

    const ctx = this.ctx;
    for (const t of overlays) {
      ctx.save();
      ctx.globalAlpha = t.opacity;
      ctx.fillStyle = t.color;
      ctx.font = `${t.fontSize}px ${t.fontFamily}`;
      ctx.textBaseline = 'top';

      if (t.align === 'center') {
        ctx.textAlign = 'center';
      } else if (t.align === 'right') {
        ctx.textAlign = 'right';
      } else {
        ctx.textAlign = 'left';
      }

      const lines = t.text.split('\n');
      const lineHeight = t.fontSize * 1.2;
      const alignX = t.align === 'center' ? t.x + t.width / 2
                    : t.align === 'right' ? t.x + t.width
                    : t.x;

      for (let i = 0; i < lines.length; i++) {
        ctx.fillText(lines[i], alignX, t.y + i * lineHeight);
      }
      ctx.restore();
    }
  }

  // Hit test at screen coordinates. Returns element ID or null.
  hitTest(screenX, screenY) {
    const id = this.engine.hit_test(screenX, screenY);
    return id || null;
  }

  // Hit test resize handles at screen coordinates.
  // Returns {id, handle} or null. handle is one of: nw, ne, sw, se
  hitTestHandle(screenX, screenY) {
    const json = this.engine.hit_test_handle(screenX, screenY);
    if (!json) return null;
    const result = JSON.parse(json);
    // Map handle names from Rust to JS convention
    const handleMap = {
      'NorthWest': 'nw', 'NorthEast': 'ne',
      'SouthWest': 'sw', 'SouthEast': 'se',
    };
    return { id: result.id, handle: handleMap[result.handle] || result.handle };
  }

  // Find element IDs in a world-space rect. Returns array of IDs.
  elementsInRect(x, y, w, h) {
    const json = this.engine.elements_in_rect(x, y, w, h);
    return JSON.parse(json);
  }

  // Convenience accessors for viewport state
  get scrollX() { return this.engine.scroll_x(); }
  set scrollX(v) {
    this.engine.set_viewport(v, this.engine.scroll_y(), this.engine.zoom());
    this.dirty = true;
  }
  get scrollY() { return this.engine.scroll_y(); }
  set scrollY(v) {
    this.engine.set_viewport(this.engine.scroll_x(), v, this.engine.zoom());
    this.dirty = true;
  }
  get zoom() { return this.engine.zoom(); }
  set zoom(v) {
    this.engine.set_viewport(this.engine.scroll_x(), this.engine.scroll_y(), v);
    this.dirty = true;
  }
}
