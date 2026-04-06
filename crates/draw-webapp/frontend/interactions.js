// Pointer event handling, hit testing, selection, drag/resize
// All element state lives in the WASM engine; JS is just event routing.

class Interactions {
  constructor(drawCanvas, app) {
    this.dc = drawCanvas;
    this.app = app;
    this.state = 'idle'; // idle, creating, moving, resizing, selecting, panning, drawing, erasing
    this.startWorld = null;
    this.startScreen = null;
    this.activeHandle = null;
    this.resizeOrigin = null;
    this.drawingPoints = [];
    this.creatingElement = null; // JSON object for element being created (preview)

    this.bind();
  }

  get engine() { return this.dc.engine; }

  bind() {
    const canvas = this.dc.canvas;
    canvas.addEventListener('pointerdown', (e) => this.onPointerDown(e));
    canvas.addEventListener('pointermove', (e) => this.onPointerMove(e));
    canvas.addEventListener('pointerup', (e) => this.onPointerUp(e));
    canvas.addEventListener('dblclick', (e) => this.onDoubleClick(e));
    canvas.addEventListener('wheel', (e) => this.onWheel(e), { passive: false });
    canvas.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      this.app.showContextMenu(e);
    });
  }

  canvasXY(e) {
    const rect = this.dc.canvas.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  // Get element JSON from engine, parsed
  getElement(id) {
    const json = this.engine.get_element(id);
    if (!json) return null;
    try { return JSON.parse(json); } catch { return null; }
  }

  // Get selection as array of IDs
  getSelection() {
    try { return JSON.parse(this.engine.get_selection()); } catch { return []; }
  }

  onPointerDown(e) {
    const screen = this.canvasXY(e);
    const world = this.dc.screenToWorld(screen.x, screen.y);
    this.startScreen = screen;
    this.startWorld = world;
    this._pointerId = e.pointerId;

    // Middle mouse or space+click = pan
    if (e.button === 1 || (e.button === 0 && this.app.spaceDown)) {
      this.dc.canvas.setPointerCapture(e.pointerId);
      this.state = 'panning';
      this.dc.canvas.style.cursor = 'grabbing';
      return;
    }

    const tool = this.app.currentTool;

    // Text tool creates an overlay — stop event so canvas doesn't steal focus
    if (tool === 'text') {
      e.preventDefault();
      e.stopPropagation();
      // Defer so the pointer event cycle completes before we focus the textarea
      requestAnimationFrame(() => this.app.startTextInput(world.x, world.y));
      return;
    }

    // Capture pointer for all other drag operations
    this.dc.canvas.setPointerCapture(e.pointerId);

    if (tool === 'select') {
      // Check if clicking on a selected element's handle
      const handleResult = this.dc.hitTestHandle(screen.x, screen.y);
      if (handleResult && this.engine.is_selected(handleResult.id)) {
        this.state = 'resizing';
        this.activeHandle = handleResult.handle;
        const el = this.getElement(handleResult.id);
        if (el) {
          this.resizeOrigin = { x: el.x, y: el.y, width: el.width, height: el.height };
          this._resizeId = handleResult.id;
        }
        return;
      }

      // Check if clicking on an element
      let hitId = this.dc.hitTest(screen.x, screen.y);
      // If hit is bound text, redirect selection to parent shape
      if (hitId) {
        const hitEl = this.getElement(hitId);
        if (hitEl && hitEl.group_id) {
          hitId = hitEl.group_id;
        }
      }
      if (hitId) {
        if (!e.shiftKey && !this.engine.is_selected(hitId)) {
          this.engine.clear_selection();
        }
        this.engine.add_to_selection(hitId);
        this.app.reflectSelectionStyle();
        this.state = 'moving';
        // Record before positions for undo
        this._moveStartWorld = { ...world };
        this.dc.markDirty();
        return;
      }

      // Click on empty = start rubber band or deselect
      this.engine.clear_selection();
      this.state = 'selecting';
      this.dc.markDirty();
      return;
    }

    if (tool === 'eraser') {
      this.state = 'erasing';
      const hitId = this.dc.hitTest(screen.x, screen.y);
      if (hitId) {
        this.app.eraseElement(hitId);
      }
      return;
    }

    if (tool === 'freedraw') {
      this.state = 'drawing';
      this.drawingPoints = [{ x: 0, y: 0 }];
      const snapped = this.app.snapPoint(world.x, world.y);
      this.creatingElement = {
        type: 'FreeDraw',
        id: generateId(),
        x: snapped.x,
        y: snapped.y,
        points: this.drawingPoints,
        stroke: this.app.currentStroke(),
        opacity: this.app.currentOpacity(),
        locked: false,
      };
      // Add as preview element to engine
      this.engine.add_element(JSON.stringify(this.creatingElement));
      this.dc.markDirty();
      return;
    }

    // Shape creation tools
    this.state = 'creating';
    const snapped = this.app.snapPoint(world.x, world.y);
    const el = this.createElementAt(tool, snapped.x, snapped.y);
    this.creatingElement = el;
    this.startWorld = snapped;
    // Add to engine as preview
    this.engine.add_element(JSON.stringify(el));
    this.dc.markDirty();
  }

  onPointerMove(e) {
    const screen = this.canvasXY(e);
    const world = this.dc.screenToWorld(screen.x, screen.y);

    if (this.state === 'panning') {
      const dx = screen.x - this.startScreen.x;
      const dy = screen.y - this.startScreen.y;
      this.dc.pan(dx, dy);
      this.startScreen = screen;
      return;
    }

    if (this.state === 'creating' && this.creatingElement) {
      const snapped = this.app.snapPoint(world.x, world.y);
      const el = this.creatingElement;
      if (el.type === 'Line' || el.type === 'Arrow') {
        el.points = [
          { x: 0, y: 0 },
          { x: snapped.x - el.x, y: snapped.y - el.y },
        ];
      } else {
        el.width = snapped.x - this.startWorld.x;
        el.height = snapped.y - this.startWorld.y;
      }
      // Update in engine: remove old, add updated
      this.engine.remove_element(el.id);
      this.engine.add_element(JSON.stringify(el));
      this.dc.markDirty();
      return;
    }

    if (this.state === 'drawing' && this.creatingElement) {
      const el = this.creatingElement;
      this.drawingPoints.push({
        x: world.x - el.x,
        y: world.y - el.y,
      });
      el.points = this.drawingPoints;
      // Update in engine
      this.engine.remove_element(el.id);
      this.engine.add_element(JSON.stringify(el));
      this.dc.markDirty();
      return;
    }

    if (this.state === 'moving') {
      const snapped = this.app.snapPoint(world.x, world.y);
      const dx = snapped.x - this._moveStartWorld.x;
      const dy = snapped.y - this._moveStartWorld.y;
      if (dx === 0 && dy === 0) return;

      const selectedIds = this.getSelection();
      const movedIds = new Set();
      for (const id of selectedIds) {
        const el = this.getElement(id);
        if (el) {
          this.engine.move_element(id, el.x + dx, el.y + dy);
          movedIds.add(id);
        }
      }
      // Move bound text elements that follow shapes
      for (const id of movedIds) {
        const btJson = this.engine.get_elements_by_group(id);
        const btIds = JSON.parse(btJson);
        for (const btId of btIds) {
          if (!movedIds.has(btId)) {
            const btEl = this.getElement(btId);
            if (btEl) {
              this.engine.move_element(btId, btEl.x + dx, btEl.y + dy);
            }
          }
        }
      }
      this._moveStartWorld = snapped;
      this.dc.markDirty();
      return;
    }

    if (this.state === 'resizing') {
      this.handleResize(world);
      return;
    }

    if (this.state === 'selecting') {
      const sx = Math.min(this.startScreen.x, screen.x);
      const sy = Math.min(this.startScreen.y, screen.y);
      const sw = Math.abs(screen.x - this.startScreen.x);
      const sh = Math.abs(screen.y - this.startScreen.y);
      this.engine.set_selection_box(sx, sy, sw, sh);
      this.dc.markDirty();
      return;
    }

    if (this.state === 'erasing') {
      const hitId = this.dc.hitTest(screen.x, screen.y);
      if (hitId) {
        this.app.eraseElement(hitId);
      }
      return;
    }

    // Update cursor based on hover and tool
    this.updateCursor(screen);
  }

  onPointerUp(e) {
    try { this.dc.canvas.releasePointerCapture(e.pointerId); } catch (_) {}

    if (this.state === 'panning') {
      const cursorMap = { select: 'default', eraser: 'crosshair', text: 'text' };
      this.dc.canvas.style.cursor = cursorMap[this.app.currentTool] || 'crosshair';
      this.state = 'idle';
      return;
    }

    if (this.state === 'creating' && this.creatingElement) {
      const el = this.creatingElement;
      // Normalize negative dimensions
      if (el.width !== undefined) {
        if (el.width < 0) { el.x += el.width; el.width = Math.abs(el.width); }
        if (el.height < 0) { el.y += el.height; el.height = Math.abs(el.height); }
        // Skip tiny accidental clicks
        if (el.width < 2 && el.height < 2) {
          this.engine.remove_element(el.id);
          // Undo the add action that was recorded
          this.engine.undo();
          this.creatingElement = null;
          this.state = 'idle';
          this.dc.markDirty();
          return;
        }
      }
      if ((el.type === 'Line' || el.type === 'Arrow') && el.points) {
        const lastPt = el.points[el.points.length - 1];
        if (Math.abs(lastPt.x) < 2 && Math.abs(lastPt.y) < 2) {
          this.engine.remove_element(el.id);
          this.engine.undo();
          this.creatingElement = null;
          this.state = 'idle';
          this.dc.markDirty();
          return;
        }
      }
      // The element is already in the engine from the preview adds.
      // Remove and re-add with final state to get a clean undo entry.
      // KNOWN ISSUE: intermediate preview states remain in undo history,
      // so Ctrl+Z shows the shape being redrawn incrementally instead of
      // a single undo step. Fix requires either batching preview adds into
      // a single Action::Batch, or storing previews outside the history.
      this.engine.remove_element(el.id);
      this.engine.add_element(JSON.stringify(el));
      this.engine.clear_selection();
      this.engine.add_to_selection(el.id);
      this.creatingElement = null;
      this.state = 'idle';
      this.app.isDirty = true;
      this.dc.markDirty();
      // Switch back to select after creating (unless tool is locked)
      if (!this.app.toolLocked) {
        this.app.setTool('select');
      }
      return;
    }

    if (this.state === 'drawing' && this.creatingElement) {
      const el = this.creatingElement;
      if (el.points.length > 1) {
        // Re-add final state
        this.engine.remove_element(el.id);
        this.engine.add_element(JSON.stringify(el));
        this.engine.clear_selection();
        this.engine.add_to_selection(el.id);
        this.app.isDirty = true;
      } else {
        this.engine.remove_element(el.id);
        this.engine.undo();
      }
      this.creatingElement = null;
      this.drawingPoints = [];
      this.state = 'idle';
      this.dc.markDirty();
      if (!this.app.toolLocked) {
        this.app.setTool('select');
      }
      return;
    }

    if (this.state === 'moving') {
      // History is tracked by engine's move_element calls
      this.app.isDirty = true;
      this.state = 'idle';
      return;
    }

    if (this.state === 'resizing') {
      if (this._resizeId) {
        const el = this.getElement(this._resizeId);
        if (el) {
          // Normalize negative dimensions
          if (el.width < 0 || el.height < 0) {
            const x = el.width < 0 ? el.x + el.width : el.x;
            const y = el.height < 0 ? el.y + el.height : el.y;
            const w = Math.abs(el.width);
            const h = Math.abs(el.height);
            this.engine.resize_element(this._resizeId, x, y, w, h);
          }
        }
      }
      this.app.isDirty = true;
      this.state = 'idle';
      this.activeHandle = null;
      this.resizeOrigin = null;
      this._resizeId = null;
      return;
    }

    if (this.state === 'erasing') {
      this.state = 'idle';
      return;
    }

    if (this.state === 'selecting') {
      // Find elements in rubber band
      const topLeft = this.dc.screenToWorld(
        Math.min(this.startScreen.x, this.canvasXY(e).x),
        Math.min(this.startScreen.y, this.canvasXY(e).y)
      );
      const botRight = this.dc.screenToWorld(
        Math.max(this.startScreen.x, this.canvasXY(e).x),
        Math.max(this.startScreen.y, this.canvasXY(e).y)
      );
      const w = botRight.x - topLeft.x;
      const h = botRight.y - topLeft.y;
      if (w > 1 || h > 1) {
        const hitIds = this.dc.elementsInRect(topLeft.x, topLeft.y, w, h);
        for (const id of hitIds) {
          // Skip bound text
          const el = this.getElement(id);
          if (el && !el.group_id) {
            this.engine.add_to_selection(id);
          }
        }
      }
      this.engine.clear_selection_box();
      this.state = 'idle';
      this.app.reflectSelectionStyle();
      this.dc.markDirty();
      return;
    }

    this.state = 'idle';
  }

  updateCursor(screen) {
    const tool = this.app.currentTool;

    if (this.app.spaceDown) {
      this.dc.canvas.style.cursor = 'grab';
      return;
    }

    if (tool === 'select') {
      // Check handles first
      const handleResult = this.dc.hitTestHandle(screen.x, screen.y);
      if (handleResult && this.engine.is_selected(handleResult.id)) {
        const cursors = { nw: 'nwse-resize', se: 'nwse-resize', ne: 'nesw-resize', sw: 'nesw-resize' };
        this.dc.canvas.style.cursor = cursors[handleResult.handle];
        return;
      }
      const hitId = this.dc.hitTest(screen.x, screen.y);
      this.dc.canvas.style.cursor = hitId ? 'move' : 'default';
      return;
    }

    if (tool === 'eraser') {
      this.dc.canvas.style.cursor = 'crosshair';
      return;
    }

    if (tool === 'text') {
      this.dc.canvas.style.cursor = 'text';
      return;
    }

    this.dc.canvas.style.cursor = 'crosshair';
  }

  onDoubleClick(e) {
    const screen = this.canvasXY(e);
    const hitId = this.dc.hitTest(screen.x, screen.y);
    if (!hitId) return;

    const hit = this.getElement(hitId);
    if (!hit) return;

    if (hit.type === 'Text') {
      this.app.editTextOnElement(hit);
    } else if (hit.type === 'Rectangle' || hit.type === 'Ellipse' || hit.type === 'Diamond') {
      this.app.editTextOnShape(hit);
    }
  }

  onWheel(e) {
    e.preventDefault();
    const screen = this.canvasXY(e);

    if (e.ctrlKey || e.metaKey) {
      const delta = -e.deltaY * 0.002;
      this.dc.setZoom(this.dc.zoom * (1 + delta), screen.x, screen.y);
      this.app.updateStatusZoom();
    } else {
      this.dc.pan(-e.deltaX, -e.deltaY);
    }
  }

  handleResize(world) {
    if (!this.resizeOrigin || !this._resizeId) return;

    const o = this.resizeOrigin;
    const handle = this.activeHandle;
    let x = o.x, y = o.y, w = o.width, h = o.height;

    switch (handle) {
      case 'se':
        w = world.x - o.x;
        h = world.y - o.y;
        break;
      case 'nw':
        x = world.x;
        y = world.y;
        w = o.x + o.width - world.x;
        h = o.y + o.height - world.y;
        break;
      case 'ne':
        y = world.y;
        w = world.x - o.x;
        h = o.y + o.height - world.y;
        break;
      case 'sw':
        x = world.x;
        w = o.x + o.width - world.x;
        h = world.y - o.y;
        break;
    }
    this.engine.resize_element(this._resizeId, x, y, w, h);
    this.dc.markDirty();
  }

  createElementAt(tool, wx, wy) {
    const id = generateId();
    const stroke = this.app.currentStroke();
    const fill = this.app.currentFill();
    const opacity = this.app.currentOpacity();

    switch (tool) {
      case 'rectangle':
        return { type: 'Rectangle', id, x: wx, y: wy, width: 0, height: 0, angle: 0, stroke, fill, opacity, locked: false, group_id: null };
      case 'ellipse':
        return { type: 'Ellipse', id, x: wx, y: wy, width: 0, height: 0, angle: 0, stroke, fill, opacity, locked: false, group_id: null };
      case 'diamond':
        return { type: 'Diamond', id, x: wx, y: wy, width: 0, height: 0, angle: 0, stroke, fill, opacity, locked: false, group_id: null };
      case 'line':
        return { type: 'Line', id, x: wx, y: wy, points: [{ x: 0, y: 0 }], stroke, start_arrowhead: null, end_arrowhead: null, opacity, locked: false, group_id: null };
      case 'arrow':
        return { type: 'Arrow', id, x: wx, y: wy, points: [{ x: 0, y: 0 }], stroke, start_arrowhead: null, end_arrowhead: "arrow", opacity, locked: false, group_id: null };
      default:
        return { type: 'Rectangle', id, x: wx, y: wy, width: 0, height: 0, angle: 0, stroke, fill, opacity, locked: false, group_id: null };
    }
  }
}

function generateId() {
  return crypto.randomUUID();
}
