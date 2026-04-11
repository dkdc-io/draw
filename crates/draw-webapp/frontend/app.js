// Main application: tool state, keyboard shortcuts, document management
// All element/document state lives in the WASM DrawEngine.
// JS is a thin event/UI shell.

class App {
  static async create() {
    await window._wasmInit;
    return new App();
  }

  constructor() {
    this.currentTool = 'select';
    this.toolLocked = false;
    this.spaceDown = false;
    this.snapToGrid = false;
    this._clipboard = [];
    this.isDirty = false;

    // Init engine + canvas
    const canvasEl = document.getElementById('canvas');
    const dpr = window.devicePixelRatio || 1;
    const rect = canvasEl.getBoundingClientRect();
    this.engine = new window.DrawEngine(
      Math.round(rect.width),
      Math.round(rect.height),
      dpr
    );
    this.dc = new DrawCanvas(canvasEl, this.engine);
    this.interactions = new Interactions(this.dc, this);

    // Toolbar
    this.initToolbar();
    this.initStyles();
    this.initKeyboard();
    this.initActions();
    this.initDrawer();

    // Auto-save every 30 seconds if dirty
    this._autoSaveTimer = setInterval(() => {
      if (this.isDirty && this.interactions.state === 'idle') this.save();
    }, THEME.autoSaveIntervalMs);

    // Load drawing if specified
    if (window.__OPEN_DRAWING_ID) {
      this.loadDrawing(window.__OPEN_DRAWING_ID);
    } else {
      this.newDocument();
    }

    this.updateStatus();

    window.addEventListener('beforeunload', (e) => {
      if (this.isDirty) {
        e.preventDefault();
      }
    });
  }

  getElement(id) { return getElement(this.engine, id); }
  getSelection() { return getSelection(this.engine); }

  initToolbar() {
    for (const btn of document.querySelectorAll('.tool-btn')) {
      btn.addEventListener('click', () => {
        const tool = btn.dataset.tool;
        if (tool === this.currentTool && tool !== 'select') {
          this.toolLocked = !this.toolLocked;
          this._updateToolStatus();
        } else {
          this.setTool(tool);
        }
      });
    }
    this.initColorPickers();
  }

  initColorPickers() {
    this._setupColorPicker('stroke-swatch', 'stroke-color', (color) => {
      this._applyStyleToSelected((el) => {
        const style = {};
        if (el.stroke) { style.stroke = { ...el.stroke, color }; }
        return style;
      });
    });

    this._setupColorPicker('fill-swatch', 'fill-color', (color) => {
      const fillStyle = document.getElementById('fill-style');
      if (color === 'transparent') {
        fillStyle.value = 'none';
        document.getElementById('no-fill').checked = true;
        document.getElementById('no-fill-btn').classList.add('active');
      } else {
        if (fillStyle.value === 'none') fillStyle.value = 'hachure';
        document.getElementById('no-fill').checked = false;
        document.getElementById('no-fill-btn').classList.remove('active');
      }
      this._applyStyleToSelected((el) => {
        const style = {};
        if (el.fill) {
          const newFill = { ...el.fill, color };
          if (color === 'transparent') {
            newFill.style = 'none';
          } else if (newFill.style === 'none') {
            newFill.style = 'hachure';
          }
          style.fill = newFill;
        }
        return style;
      });
    });

    // Sync swatch background colors on input change
    document.getElementById('stroke-color').addEventListener('input', () => {
      document.getElementById('stroke-swatch').style.background =
        document.getElementById('stroke-color').value;
    });
    document.getElementById('fill-color').addEventListener('input', () => {
      document.getElementById('fill-swatch').style.background =
        document.getElementById('fill-color').value;
    });

    // Set initial swatch colors
    document.getElementById('stroke-swatch').style.background =
      document.getElementById('stroke-color').value;
    document.getElementById('fill-swatch').style.background =
      document.getElementById('fill-color').value;
  }

  _setupColorPicker(swatchId, inputId, onColorChange) {
    const swatch = document.getElementById(swatchId);
    const input = document.getElementById(inputId);

    swatch.addEventListener('click', (e) => {
      e.stopPropagation();
      document.querySelectorAll('.color-palette').forEach(p => p.remove());

      const palette = document.createElement('div');
      palette.className = 'color-palette';

      for (const color of THEME.palette) {
        const dot = document.createElement('button');
        dot.className = 'color-palette-dot';
        if (color === 'transparent') {
          dot.classList.add('transparent-dot');
        } else {
          dot.style.background = color;
        }
        if (input.value === color) dot.classList.add('active');

        dot.addEventListener('click', (e) => {
          e.stopPropagation();
          if (color !== 'transparent') {
            input.value = color;
            swatch.style.background = color;
          }
          onColorChange(color);
          palette.remove();
        });
        palette.appendChild(dot);
      }

      const customRow = document.createElement('div');
      customRow.className = 'color-palette-custom';
      const customBtn = document.createElement('button');
      customBtn.className = 'color-palette-custom-btn';
      customBtn.textContent = 'Custom...';
      customBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        palette.remove();
        input.click();
      });
      customRow.appendChild(customBtn);
      palette.appendChild(customRow);

      swatch.parentElement.appendChild(palette);

      const close = () => {
        palette.remove();
        document.removeEventListener('click', close);
      };
      setTimeout(() => document.addEventListener('click', close), 0);
    });
  }

  // Apply a style update to all selected elements via engine
  _applyStyleToSelected(buildStyleFn) {
    const selectedIds = this.getSelection();
    if (selectedIds.length === 0) return;

    for (const id of selectedIds) {
      const el = this.getElement(id);
      if (!el) continue;
      const styleUpdate = buildStyleFn(el);
      if (styleUpdate && Object.keys(styleUpdate).length > 0) {
        this.engine.update_element_style(id, JSON.stringify(styleUpdate));
      }
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  initStyles() {
    // Quick-access color swatches
    const quickColors = ['#e2e8f0', '#3b82f6', '#ef4444', '#34d399', '#f59e0b', '#a855f7'];
    for (const wrap of document.querySelectorAll('.quick-colors')) {
      const target = wrap.dataset.target; // 'stroke' or 'fill'
      for (const color of quickColors) {
        const dot = document.createElement('span');
        dot.className = 'quick-color-dot';
        dot.style.background = color;
        dot.title = color;
        dot.addEventListener('click', () => {
          document.getElementById(`${target}-color`).value = color;
          document.getElementById(`${target}-swatch`).style.background = color;
          this._applyStyleToSelected((el) => {
            if (target === 'stroke' && el.stroke) return { stroke: { ...el.stroke, color } };
            if (target === 'fill' && el.fill) {
              const newFill = { ...el.fill, color };
              if (newFill.style === 'none') newFill.style = 'hachure';
              return { fill: newFill };
            }
            return {};
          });
        });
        wrap.appendChild(dot);
      }
    }

    document.getElementById('stroke-color').addEventListener('input', (e) => {
      this._applyStyleToSelected((el) => {
        if (el.stroke) return { stroke: { ...el.stroke, color: e.target.value } };
        return {};
      });
    });

    document.getElementById('fill-color').addEventListener('input', (e) => {
      this._applyStyleToSelected((el) => {
        if (el.fill) {
          const newFill = { ...el.fill, color: e.target.value };
          if (newFill.style === 'none') newFill.style = 'hachure';
          return { fill: newFill };
        }
        return {};
      });
      const fillStyleSelect = document.getElementById('fill-style');
      if (fillStyleSelect.value === 'none') fillStyleSelect.value = 'hachure';
      document.getElementById('no-fill').checked = false;
      document.getElementById('no-fill-btn').classList.remove('active');
    });

    document.getElementById('fill-style').addEventListener('change', (e) => {
      const style = e.target.value;
      const isNone = style === 'none';
      document.getElementById('no-fill').checked = isNone;
      document.getElementById('no-fill-btn').classList.toggle('active', isNone);
      this._applyStyleToSelected((el) => {
        if (el.fill) {
          const newFill = { ...el.fill, style };
          if (isNone) {
            newFill.color = 'transparent';
          } else if (newFill.color === 'transparent') {
            newFill.color = document.getElementById('fill-color').value;
          }
          return { fill: newFill };
        }
        return {};
      });
    });

    document.getElementById('fill-density').addEventListener('change', (e) => {
      const gap = parseFloat(e.target.value);
      this._applyStyleToSelected((el) => {
        if (el.fill) return { fill: { ...el.fill, gap } };
        return {};
      });
    });

    document.getElementById('no-fill').addEventListener('change', (e) => {
      const fillStyleSelect = document.getElementById('fill-style');
      if (e.target.checked) {
        fillStyleSelect.value = 'none';
      } else {
        fillStyleSelect.value = 'hachure';
      }
      fillStyleSelect.dispatchEvent(new Event('change'));
    });

    document.getElementById('stroke-width').addEventListener('change', (e) => {
      this._applyStyleToSelected((el) => {
        if (el.stroke) return { stroke: { ...el.stroke, width: parseFloat(e.target.value) } };
        return {};
      });
    });

    document.getElementById('font-size').addEventListener('change', (e) => {
      const newSize = parseInt(e.target.value);
      const selectedIds = this.getSelection();
      for (const id of selectedIds) {
        const el = this.getElement(id);
        if (el?.font) {
          this.engine.update_element_style(id, JSON.stringify({ font: { ...el.font, size: newSize } }));
        }
        // Also update bound text
        const btIds = JSON.parse(this.engine.get_elements_by_group(id));
        for (const btId of btIds) {
          const bt = this.getElement(btId);
          if (bt?.font) {
            this.engine.update_element_style(btId, JSON.stringify({ font: { ...bt.font, size: newSize } }));
          }
        }
      }
      this.isDirty = true;
      this.updateStatusName();
      this.dc.markDirty();
    });

    document.getElementById('opacity').addEventListener('input', (e) => {
      document.getElementById('opacity-value').textContent = Math.round(parseFloat(e.target.value) * 100) + '%';
      this._applyStyleToSelected(() => ({ opacity: parseFloat(e.target.value) }));
    });

    document.getElementById('stroke-dash').addEventListener('change', (e) => {
      const dashVal = e.target.value;
      const dash = dashVal ? dashVal.split(',').map(Number) : [];
      this._applyStyleToSelected((el) => {
        if (el.stroke) return { stroke: { ...el.stroke, dash } };
        return {};
      });
    });
  }

  reflectSelectionStyle() {
    const selectedIds = this.getSelection();
    if (selectedIds.length !== 1) return;
    const el = this.getElement(selectedIds[0]);
    if (!el) return;

    if (el.stroke?.color) {
      document.getElementById('stroke-color').value = el.stroke.color;
      document.getElementById('stroke-swatch').style.background = el.stroke.color;
    }
    if (el.fill) {
      const isNone = !el.fill.color || el.fill.color === 'transparent' || el.fill.style === 'none';
      document.getElementById('no-fill').checked = isNone;
      document.getElementById('no-fill-btn').classList.toggle('active', isNone);
      document.getElementById('fill-style').value = el.fill.style || 'none';
      if (!isNone && el.fill.color) {
        document.getElementById('fill-color').value = el.fill.color;
        document.getElementById('fill-swatch').style.background = el.fill.color;
      }
      if (el.fill.gap) {
        const densitySelect = document.getElementById('fill-density');
        const closest = [...densitySelect.options].reduce((prev, opt) =>
          Math.abs(parseFloat(opt.value) - el.fill.gap) < Math.abs(parseFloat(prev.value) - el.fill.gap) ? opt : prev
        );
        densitySelect.value = closest.value;
      }
    }
    if (el.stroke?.width) {
      const widthSelect = document.getElementById('stroke-width');
      const closest = [...widthSelect.options].reduce((prev, opt) =>
        Math.abs(parseFloat(opt.value) - el.stroke.width) < Math.abs(parseFloat(prev.value) - el.stroke.width) ? opt : prev
      );
      widthSelect.value = closest.value;
    }
    if (el.font?.size) {
      const sizeSelect = document.getElementById('font-size');
      const closest = [...sizeSelect.options].reduce((prev, opt) =>
        Math.abs(parseInt(opt.value) - el.font.size) < Math.abs(parseInt(prev.value) - el.font.size) ? opt : prev
      );
      sizeSelect.value = closest.value;
    }
    if (el.opacity != null) {
      document.getElementById('opacity').value = el.opacity;
      document.getElementById('opacity-value').textContent = Math.round(el.opacity * 100) + '%';
    }
    if (el.stroke?.dash) {
      const dashStr = el.stroke.dash.join(',');
      const dashSelect = document.getElementById('stroke-dash');
      let matched = false;
      for (const opt of dashSelect.options) {
        if (opt.value === dashStr) {
          dashSelect.value = opt.value;
          matched = true;
          break;
        }
      }
      if (!matched) dashSelect.value = '';
    }

    // Reflect bound text font info
    const btIds = JSON.parse(this.engine.get_elements_by_group(el.id));
    if (btIds.length > 0) {
      const bt = this.getElement(btIds[0]);
      if (bt?.font?.size) {
        const sizeSelect = document.getElementById('font-size');
        const closest = [...sizeSelect.options].reduce((prev, opt) =>
          Math.abs(parseInt(opt.value) - bt.font.size) < Math.abs(parseInt(prev.value) - bt.font.size) ? opt : prev
        );
        sizeSelect.value = closest.value;
      }
    }
  }

  initKeyboard() {
    document.addEventListener('keydown', (e) => {
      if (e.target.tagName === 'TEXTAREA' || e.target.tagName === 'INPUT') return;

      if (e.key === ' ') {
        e.preventDefault();
        this.spaceDown = true;
        this.dc.canvas.style.cursor = 'grab';
        return;
      }

      const toolKeys = {
        v: 'select', r: 'rectangle', o: 'ellipse', d: 'diamond',
        l: 'line', a: 'arrow', p: 'freedraw', t: 'text', e: 'eraser',
      };
      if (!e.ctrlKey && !e.metaKey && toolKeys[e.key.toLowerCase()]) {
        this.setTool(toolKeys[e.key.toLowerCase()]);
        return;
      }

      // Grid snap toggle
      if (e.key === 'g' && !e.ctrlKey && !e.metaKey) {
        this.snapToGrid = !this.snapToGrid;
        this._updateToolStatus();
        return;
      }

      // Arrow keys to nudge selected elements
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        const selectedIds = this.getSelection();
        if (selectedIds.length > 0) {
          e.preventDefault();
          const step = e.shiftKey ? 10 : 1;
          let dx = 0, dy = 0;
          if (e.key === 'ArrowLeft') dx = -step;
          if (e.key === 'ArrowRight') dx = step;
          if (e.key === 'ArrowUp') dy = -step;
          if (e.key === 'ArrowDown') dy = step;

          const movedIds = new Set();
          for (const id of selectedIds) {
            const el = this.getElement(id);
            if (el) {
              this.engine.move_element(id, el.x + dx, el.y + dy);
              movedIds.add(id);
            }
          }
          // Move bound text
          for (const id of movedIds) {
            const btIds = JSON.parse(this.engine.get_elements_by_group(id));
            for (const btId of btIds) {
              if (!movedIds.has(btId)) {
                const bt = this.getElement(btId);
                if (bt) this.engine.move_element(btId, bt.x + dx, bt.y + dy);
              }
            }
          }
          this.isDirty = true;
          this.dc.markDirty();
          return;
        }
      }

      // Copy
      if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
        const selectedIds = this.getSelection();
        if (selectedIds.length > 0) {
          e.preventDefault();
          this.copySelected();
        }
        return;
      }

      // Paste
      if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
        e.preventDefault();
        this.pasteClipboard(null);
        return;
      }

      // Bring to front / send to back
      if (e.key === ']' && !e.ctrlKey && !e.metaKey && this.getSelection().length > 0) {
        this.bringToFront();
        return;
      }
      if (e.key === '[' && !e.ctrlKey && !e.metaKey && this.getSelection().length > 0) {
        this.sendToBack();
        return;
      }

      // Undo/Redo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
          this.engine.redo();
        } else {
          this.engine.undo();
        }
        this.dc.markDirty();
        this.updateStatusName();
        return;
      }

      // Zoom shortcuts
      if ((e.ctrlKey || e.metaKey) && (e.key === '0' || e.key === '=')) {
        e.preventDefault();
        const cx = this.dc.width / 2;
        const cy = this.dc.height / 2;
        this.dc.setZoom(1, cx, cy);
        this.dc.scrollX = 0;
        this.dc.scrollY = 0;
        this.dc.markDirty();
        this.updateStatusZoom();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '+') {
        e.preventDefault();
        const cx = this.dc.width / 2;
        const cy = this.dc.height / 2;
        this.dc.setZoom(this.dc.zoom * 1.25, cx, cy);
        this.updateStatusZoom();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '-') {
        e.preventDefault();
        const cx = this.dc.width / 2;
        const cy = this.dc.height / 2;
        this.dc.setZoom(this.dc.zoom / 1.25, cx, cy);
        this.updateStatusZoom();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '1') {
        e.preventDefault();
        this.zoomToFit();
        return;
      }

      // Save
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        this.save();
        return;
      }

      // Delete
      if (e.key === 'Delete' || e.key === 'Backspace') {
        e.preventDefault();
        this.deleteSelected();
        return;
      }

      // Select all
      if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
        e.preventDefault();
        this.engine.select_all();
        this.dc.markDirty();
        return;
      }

      // Duplicate
      if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
        e.preventDefault();
        this.duplicateSelected();
        return;
      }

      // Help overlay
      if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
        this.toggleHelp();
        return;
      }

      // Escape
      if (e.key === 'Escape') {
        const help = document.getElementById('help-overlay');
        if (help) { help.remove(); return; }
        this.engine.clear_selection();
        this.interactions.creatingElement = null;
        this.setTool('select');
        this.dc.markDirty();
        return;
      }
    });

    document.addEventListener('keyup', (e) => {
      if (e.key === ' ') {
        this.spaceDown = false;
        const cursorMap = { select: 'default', eraser: 'crosshair', text: 'text' };
        this.dc.canvas.style.cursor = cursorMap[this.currentTool] || 'crosshair';
      }
    });
  }

  initActions() {
    document.getElementById('btn-save').addEventListener('click', () => this.save());
    document.getElementById('btn-export').addEventListener('click', () => this.exportSvg());
    document.getElementById('btn-export-png').addEventListener('click', () => this.exportPng());

    document.getElementById('btn-undo').addEventListener('click', () => {
      this.engine.undo();
      this.dc.markDirty();
      this.updateStatusName();
    });
    document.getElementById('btn-redo').addEventListener('click', () => {
      this.engine.redo();
      this.dc.markDirty();
      this.updateStatusName();
    });

    document.getElementById('no-fill-btn').addEventListener('click', () => {
      const cb = document.getElementById('no-fill');
      cb.checked = !cb.checked;
      cb.dispatchEvent(new Event('change'));
    });

    document.getElementById('status-name').addEventListener('click', () => {
      this._startInlineRename();
    });

    document.getElementById('zoom-btn').addEventListener('click', () => {
      this.dc.zoom = 1;
      this.dc.scrollX = 0;
      this.dc.scrollY = 0;
      this.dc.markDirty();
      this.updateStatusZoom();
    });
  }

  // ── Drawing browser drawer ──────────────────────────────────────

  initDrawer() {
    const drawer = document.getElementById('drawer');
    document.getElementById('drawer-toggle').addEventListener('click', () => {
      const opening = !drawer.classList.contains('open');
      drawer.classList.toggle('open');
      if (opening) this.refreshDrawerList();
    });
    document.getElementById('drawer-close').addEventListener('click', () => {
      drawer.classList.remove('open');
    });
    document.getElementById('drawer-new').addEventListener('click', async () => {
      if (this.isDirty) await this.save();
      this.newDocument();
      this.refreshDrawerList();
    });
  }

  async refreshDrawerList() {
    const list = document.getElementById('drawer-list');
    try {
      const drawings = await API.listDrawings();
      const currentId = this.engine.document_id();
      if (drawings.length === 0) {
        list.innerHTML = '<div class="drawer-empty">No saved drawings yet</div>';
        return;
      }
      list.innerHTML = '';
      for (const d of drawings) {
        const item = document.createElement('div');
        item.className = 'drawer-item' + (d.id === currentId ? ' active' : '');
        const name = document.createElement('span');
        name.className = 'drawer-item-name';
        name.textContent = d.name || d.id;
        const del = document.createElement('button');
        del.className = 'drawer-item-delete';
        del.title = 'Delete';
        del.textContent = '\u00d7';
        del.addEventListener('click', async (e) => {
          e.stopPropagation();
          if (!confirm(`Delete "${d.name || d.id}"?`)) return;
          try {
            await API.deleteDrawing(d.id);
            if (d.id === this.engine.document_id()) this.newDocument();
            this.refreshDrawerList();
          } catch (err) {
            console.error('Delete failed:', err);
          }
        });
        item.addEventListener('click', async () => {
          if (d.id === currentId) return;
          if (this.isDirty) await this.save();
          await this.loadDrawing(d.id);
          document.getElementById('drawer').classList.remove('open');
        });
        item.appendChild(name);
        item.appendChild(del);
        list.appendChild(item);
      }
    } catch (e) {
      list.innerHTML = '<div class="drawer-empty">Failed to load drawings</div>';
    }
  }

  snapPoint(x, y) {
    if (!this.snapToGrid) return { x, y };
    const gs = this.dc.gridSize;
    return {
      x: Math.round(x / gs) * gs,
      y: Math.round(y / gs) * gs,
    };
  }

  setTool(tool) {
    if (tool !== this.currentTool) {
      this.toolLocked = false;
    }
    this.currentTool = tool;
    for (const btn of document.querySelectorAll('.tool-btn')) {
      btn.classList.toggle('active', btn.dataset.tool === tool);
    }
    const cursorMap = {
      select: 'default',
      eraser: 'crosshair',
      text: 'text',
    };
    this.dc.canvas.style.cursor = cursorMap[tool] || 'crosshair';
    this._updateToolStatus();
  }

  _updateToolStatus() {
    const label = this.currentTool.charAt(0).toUpperCase() + this.currentTool.slice(1);
    const suffix = this.toolLocked ? ' [locked]' : '';
    const sel = this.getSelection();
    const selSuffix = sel.length > 1 ? ` \u00b7 ${sel.length} selected` : '';
    document.getElementById('status-tool').textContent = label + suffix + selSuffix;
    const snapBadge = document.getElementById('snap-badge');
    if (snapBadge) snapBadge.style.display = this.snapToGrid ? '' : 'none';
  }

  showContextMenu(e) {
    document.querySelectorAll('.context-menu').forEach(m => m.remove());

    const screen = this.interactions.canvasXY(e);
    const world = this.dc.screenToWorld(screen.x, screen.y);
    let hitId = this.dc.hitTest(screen.x, screen.y);

    // Redirect bound text to parent
    if (hitId) {
      const hitEl = this.getElement(hitId);
      if (hitEl && hitEl.group_id) {
        hitId = hitEl.group_id;
      }
    }

    // If right-clicking an unselected element, select it
    if (hitId && !this.engine.is_selected(hitId)) {
      this.engine.clear_selection();
      this.engine.add_to_selection(hitId);
      this.dc.markDirty();
    }

    const selectedIds = this.getSelection();
    const hasSelection = selectedIds.length > 0;
    const hasClipboard = this._clipboard && this._clipboard.length > 0;

    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.style.left = e.clientX + 'px';
    menu.style.top = e.clientY + 'px';

    const items = [];

    if (hasSelection) {
      items.push({ label: 'Copy', shortcut: '\u2318C', action: () => this.copySelected() });
      items.push({ label: 'Paste', shortcut: '\u2318V', action: () => this.pasteClipboard(world), disabled: !hasClipboard });
      items.push({ label: 'Duplicate', shortcut: '\u2318D', action: () => this.duplicateSelected() });
      items.push({ type: 'separator' });
      items.push({ label: 'Bring to front', shortcut: ']', action: () => this.bringToFront() });
      items.push({ label: 'Send to back', shortcut: '[', action: () => this.sendToBack() });
      items.push({ label: 'Bring forward', shortcut: '', action: () => this.bringForward() });
      items.push({ label: 'Send backward', shortcut: '', action: () => this.sendBackward() });
      items.push({ type: 'separator' });
      items.push({ label: 'Delete', shortcut: 'Del', action: () => this.deleteSelected() });
    } else {
      items.push({ label: 'Paste', shortcut: '\u2318V', action: () => this.pasteClipboard(world), disabled: !hasClipboard });
      items.push({ type: 'separator' });
      items.push({ label: 'Select all', shortcut: '\u2318A', action: () => {
        this.engine.select_all();
        this.dc.markDirty();
      }});
    }

    for (const item of items) {
      if (item.type === 'separator') {
        const sep = document.createElement('div');
        sep.className = 'context-menu-sep';
        menu.appendChild(sep);
        continue;
      }
      const row = document.createElement('button');
      row.className = 'context-menu-item';
      if (item.disabled) row.disabled = true;

      const label = document.createElement('span');
      label.textContent = item.label;
      row.appendChild(label);

      if (item.shortcut) {
        const shortcut = document.createElement('span');
        shortcut.className = 'context-menu-shortcut';
        shortcut.textContent = item.shortcut;
        row.appendChild(shortcut);
      }

      row.addEventListener('click', () => {
        menu.remove();
        item.action();
      });
      menu.appendChild(row);
    }

    document.body.appendChild(menu);

    const close = () => {
      menu.remove();
      document.removeEventListener('click', close);
    };
    setTimeout(() => document.addEventListener('click', close), 0);
  }

  // Z-ordering

  bringToFront() {
    const ids = this.getSelection();
    for (const id of ids) {
      this.engine.reorder_to_front(id);
      // Also move bound text
      const btIds = JSON.parse(this.engine.get_elements_by_group(id));
      for (const btId of btIds) {
        this.engine.reorder_to_front(btId);
      }
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  sendToBack() {
    const ids = [...this.getSelection()].reverse();
    for (const id of ids) {
      const btIds = JSON.parse(this.engine.get_elements_by_group(id));
      for (const btId of btIds) {
        this.engine.reorder_to_back(btId);
      }
      this.engine.reorder_to_back(id);
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  bringForward() {
    const ids = this.getSelection();
    for (const id of ids) {
      this.engine.reorder_forward(id);
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  sendBackward() {
    const ids = this.getSelection();
    for (const id of ids) {
      this.engine.reorder_backward(id);
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  // Copy/paste

  copySelected() {
    this._clipboard = [];
    const ids = this.getSelection();
    for (const id of ids) {
      const el = this.getElement(id);
      if (el) this._clipboard.push(structuredClone(el));
    }
    // Copy bound text for selected shapes
    for (const id of ids) {
      const btIds = JSON.parse(this.engine.get_elements_by_group(id));
      for (const btId of btIds) {
        if (!ids.includes(btId)) {
          const bt = this.getElement(btId);
          if (bt) this._clipboard.push(structuredClone(bt));
        }
      }
    }
  }

  pasteClipboard(worldPos) {
    if (!this._clipboard?.length) return;

    let cx = 0, cy = 0, count = 0;
    for (const el of this._clipboard) {
      if (!el.group_id) {
        cx += el.x + (el.width || 0) / 2;
        cy += el.y + (el.height || 0) / 2;
        count++;
      }
    }
    if (count > 0) { cx /= count; cy /= count; }

    const dx = worldPos ? worldPos.x - cx : THEME.pasteOffset;
    const dy = worldPos ? worldPos.y - cy : THEME.pasteOffset;

    const idMap = new Map();
    const shapesFirst = this._clipboard.filter(e => !e.group_id);
    const boundText = this._clipboard.filter(e => e.group_id);
    const ordered = [...shapesFirst, ...boundText];

    this.engine.clear_selection();
    for (const orig of ordered) {
      const dup = structuredClone(orig);
      const newId = generateId();
      idMap.set(orig.id, newId);
      dup.id = newId;
      dup.x += dx;
      dup.y += dy;
      if (dup.group_id && idMap.has(dup.group_id)) {
        dup.group_id = idMap.get(dup.group_id);
      }
      this.engine.add_element(JSON.stringify(dup));
      if (!dup.group_id) {
        this.engine.add_to_selection(dup.id);
      }
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  toggleHelp() {
    let overlay = document.getElementById('help-overlay');
    if (overlay) {
      overlay.remove();
      return;
    }
    overlay = document.createElement('div');
    overlay.id = 'help-overlay';
    overlay.innerHTML = `
      <div class="help-content">
        <h3>Keyboard Shortcuts</h3>
        <div class="help-grid">
          <div><kbd>V</kbd> Select</div>
          <div><kbd>R</kbd> Rectangle</div>
          <div><kbd>O</kbd> Ellipse</div>
          <div><kbd>D</kbd> Diamond</div>
          <div><kbd>L</kbd> Line</div>
          <div><kbd>A</kbd> Arrow</div>
          <div><kbd>P</kbd> Pen</div>
          <div><kbd>T</kbd> Text</div>
          <div><kbd>E</kbd> Eraser</div>
          <div><kbd>G</kbd> Grid snap</div>
          <div><kbd>?</kbd> This help</div>
          <div><kbd>Esc</kbd> Deselect</div>
          <div><kbd>Del</kbd> Delete</div>
          <div><kbd>\u2318Z</kbd> Undo</div>
          <div><kbd>\u2318\u21e7Z</kbd> Redo</div>
          <div><kbd>\u2318S</kbd> Save</div>
          <div><kbd>\u2318A</kbd> Select all</div>
          <div><kbd>\u2318D</kbd> Duplicate</div>
          <div><kbd>\u2318C</kbd> Copy</div>
          <div><kbd>\u2318V</kbd> Paste</div>
          <div><kbd>]</kbd> Bring to front</div>
          <div><kbd>[</kbd> Send to back</div>
          <div><kbd>\u23180</kbd> Reset zoom</div>
          <div><kbd>\u23181</kbd> Zoom to fit</div>
          <div><kbd>\u2191\u2193\u2190\u2192</kbd> Nudge</div>
          <div><kbd>Space</kbd> Pan</div>
          <div><kbd>Dbl-click</kbd> Edit text</div>
        </div>
        <p class="help-dismiss">Press <kbd>?</kbd> or <kbd>Esc</kbd> to close</p>
      </div>
    `;
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) overlay.remove();
    });
    document.body.appendChild(overlay);
  }

  currentStroke() {
    const dashVal = document.getElementById('stroke-dash').value;
    const dash = dashVal ? dashVal.split(',').map(Number) : [];
    return {
      color: document.getElementById('stroke-color').value,
      width: parseFloat(document.getElementById('stroke-width').value),
      dash,
    };
  }

  currentOpacity() {
    return parseFloat(document.getElementById('opacity').value);
  }

  currentFill() {
    const style = document.getElementById('fill-style').value;
    if (style === 'none') {
      return { color: 'transparent', style: 'none', gap: THEME.hachureGap, angle: THEME.hachureAngle };
    }
    return {
      color: document.getElementById('fill-color').value,
      style,
      gap: parseFloat(document.getElementById('fill-density').value),
      angle: THEME.hachureAngle,
    };
  }

  addElement(element) {
    this.engine.add_element(JSON.stringify(element));
    this.isDirty = true;
    this.updateStatusName();
    this.dc.markDirty();
  }

  removeElement(id) {
    this.engine.remove_element(id);
    this.isDirty = true;
    this.updateStatusName();
    this.dc.markDirty();
  }

  eraseElement(id) {
    const target = this.getElement(id);
    if (!target) return;
    // Redirect bound text to parent
    let targetId = id;
    if (target.group_id) {
      targetId = target.group_id;
    }
    // Remove bound text of this shape
    const btIds = JSON.parse(this.engine.get_elements_by_group(targetId));
    for (const btId of btIds) {
      this.engine.remove_element(btId);
    }
    this.engine.remove_element(targetId);
    this.isDirty = true;
    this.dc.markDirty();
  }

  deleteSelected() {
    const ids = this.getSelection();
    // Collect bound text ids for shapes being deleted
    const allIds = [...ids];
    for (const id of ids) {
      const btIds = JSON.parse(this.engine.get_elements_by_group(id));
      for (const btId of btIds) {
        if (!allIds.includes(btId)) {
          allIds.push(btId);
        }
      }
    }
    this.engine.remove_elements(JSON.stringify(allIds));
    this.isDirty = true;
    this.updateStatusName();
    this.dc.markDirty();
  }

  duplicateSelected() {
    const ids = this.getSelection();
    const idMap = new Map();

    this.engine.clear_selection();
    for (const id of ids) {
      const el = this.getElement(id);
      if (el) {
        const dup = structuredClone(el);
        const newId = generateId();
        idMap.set(el.id, newId);
        dup.id = newId;
        dup.x += THEME.pasteOffset;
        dup.y += THEME.pasteOffset;
        this.engine.add_element(JSON.stringify(dup));
        this.engine.add_to_selection(dup.id);
      }
    }
    // Duplicate bound text for duplicated shapes
    for (const [oldId, newId] of idMap) {
      const btIds = JSON.parse(this.engine.get_elements_by_group(oldId));
      for (const btId of btIds) {
        if (!idMap.has(btId)) {
          const bt = this.getElement(btId);
          if (bt) {
            const dupText = structuredClone(bt);
            dupText.id = generateId();
            dupText.x += THEME.pasteOffset;
            dupText.y += THEME.pasteOffset;
            dupText.group_id = newId;
            this.engine.add_element(JSON.stringify(dupText));
          }
        }
      }
    }
    this.isDirty = true;
    this.dc.markDirty();
  }

  findBoundText(shapeId) {
    const btIds = JSON.parse(this.engine.get_elements_by_group(shapeId));
    if (btIds.length > 0) {
      return this.getElement(btIds[0]);
    }
    return null;
  }

  centerTextInShape(textEl, shapeEl) {
    // Approximate bounds for centering
    const fontSize = textEl.font?.size || 20;
    const lines = textEl.text.split('\n');
    const textHeight = lines.length * fontSize * 1.2;
    const bx = shapeEl.width < 0 ? shapeEl.x + shapeEl.width : shapeEl.x;
    const by = shapeEl.height < 0 ? shapeEl.y + shapeEl.height : shapeEl.y;
    const bw = Math.abs(shapeEl.width || 0);
    const bh = Math.abs(shapeEl.height || 0);
    const newX = bx + bw / 2;
    const newY = by + (bh - textHeight) / 2;
    this.engine.move_element(textEl.id, newX, newY);
  }

  editTextOnShape(shapeEl) {
    const existing = this.findBoundText(shapeEl.id);
    if (existing) {
      this.editTextOnElement(existing, shapeEl);
    } else {
      this._openTextEditor({
        shapeEl,
        existingTextEl: null,
        centered: true,
      });
    }
  }

  editTextOnElement(textEl, parentShape) {
    if (!parentShape && textEl.group_id) {
      parentShape = this.getElement(textEl.group_id);
    }
    this._openTextEditor({
      shapeEl: parentShape,
      existingTextEl: textEl,
      centered: !!parentShape,
    });
  }

  _openTextEditor({ shapeEl, existingTextEl, centered, worldX, worldY }) {
    document.querySelectorAll('.text-input-overlay').forEach(el => el.remove());

    const fontSize = existingTextEl
      ? existingTextEl.font?.size || 20
      : parseInt(document.getElementById('font-size').value);

    let screenX, screenY, overlayWidth, overlayHeight;
    if (centered && shapeEl) {
      const bx = shapeEl.width < 0 ? shapeEl.x + shapeEl.width : shapeEl.x;
      const by = shapeEl.height < 0 ? shapeEl.y + shapeEl.height : shapeEl.y;
      const bw = Math.abs(shapeEl.width || 0);
      const bh = Math.abs(shapeEl.height || 0);
      const topLeft = this.dc.worldToScreen(bx, by);
      const botRight = this.dc.worldToScreen(bx + bw, by + bh);
      screenX = topLeft.x;
      screenY = topLeft.y;
      overlayWidth = botRight.x - topLeft.x;
      overlayHeight = botRight.y - topLeft.y;
    } else if (existingTextEl) {
      const screen = this.dc.worldToScreen(existingTextEl.x, existingTextEl.y);
      screenX = screen.x;
      screenY = screen.y;
    } else if (worldX != null && worldY != null) {
      const screen = this.dc.worldToScreen(worldX, worldY);
      screenX = screen.x;
      screenY = screen.y;
    }

    const overlay = document.createElement('div');
    overlay.className = 'text-input-overlay';
    if (centered && shapeEl) {
      overlay.classList.add('text-input-centered');
      overlay.style.left = screenX + 'px';
      overlay.style.top = screenY + 'px';
      overlay.style.width = overlayWidth + 'px';
      overlay.style.height = overlayHeight + 'px';
    } else {
      overlay.style.left = screenX + 'px';
      overlay.style.top = screenY + 'px';
    }

    const textarea = document.createElement('textarea');
    textarea.style.fontSize = fontSize * this.dc.zoom + 'px';
    textarea.style.color = existingTextEl
      ? existingTextEl.stroke?.color || THEME.strokeColor
      : document.getElementById('stroke-color').value;
    if (centered) {
      textarea.style.textAlign = 'center';
    }

    if (existingTextEl) {
      textarea.value = existingTextEl.text;
    }

    overlay.appendChild(textarea);
    document.body.appendChild(overlay);
    textarea.focus();
    textarea.select();

    const commit = () => {
      const text = textarea.value.trim();
      if (existingTextEl) {
        if (text) {
          if (text !== existingTextEl.text) {
            // Update via engine
            this.engine.update_element_style(existingTextEl.id, JSON.stringify({ text }));
            if (shapeEl) {
              const updatedText = this.getElement(existingTextEl.id);
              if (updatedText) this.centerTextInShape(updatedText, shapeEl);
            }
            this.isDirty = true;
            this.updateStatusName();
            this.dc.markDirty();
          }
        } else {
          this.removeElement(existingTextEl.id);
        }
      } else if (text) {
        const el = {
          type: 'Text',
          id: generateId(),
          x: worldX || 0,
          y: worldY || 0,
          text,
          font: {
            family: THEME.fontFamily,
            size: fontSize,
            align: shapeEl ? 'center' : 'left',
          },
          opacity: this.currentOpacity(),
          angle: 0,
          locked: false,
          group_id: shapeEl ? shapeEl.id : null,
          stroke: this.currentStroke(),
        };
        if (shapeEl) {
          // Compute center position
          const bx = shapeEl.width < 0 ? shapeEl.x + shapeEl.width : shapeEl.x;
          const by = shapeEl.height < 0 ? shapeEl.y + shapeEl.height : shapeEl.y;
          const bw = Math.abs(shapeEl.width || 0);
          const bh = Math.abs(shapeEl.height || 0);
          const lines = text.split('\n');
          const textHeight = lines.length * fontSize * 1.2;
          el.x = bx + bw / 2;
          el.y = by + (bh - textHeight) / 2;
        }
        this.engine.add_element(JSON.stringify(el));
        this.isDirty = true;
        this.updateStatusName();
        this.dc.markDirty();
      }
      overlay.remove();
      this.setTool('select');
    };

    // Defer blur handler so it doesn't fire during the initial focus
    setTimeout(() => textarea.addEventListener('blur', commit), 100);
    textarea.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        textarea.removeEventListener('blur', commit);
        overlay.remove();
        this.setTool('select');
      }
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        commit();
      }
    });
  }

  startTextInput(wx, wy) {
    this._openTextEditor({
      shapeEl: null,
      existingTextEl: null,
      centered: false,
      worldX: wx,
      worldY: wy,
    });
  }

  zoomToFit() {
    if (this.engine.element_count() === 0) return;
    // Get all elements and compute bounds
    const allIds = JSON.parse(this.engine.get_all_element_ids());
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const id of allIds) {
      const el = this.getElement(id);
      if (!el) continue;
      const bx = el.x;
      const by = el.y;
      const bw = el.width || 0;
      const bh = el.height || 0;
      minX = Math.min(minX, bx);
      minY = Math.min(minY, by);
      maxX = Math.max(maxX, bx + bw);
      maxY = Math.max(maxY, by + bh);
    }
    const contentWidth = maxX - minX;
    const contentHeight = maxY - minY;
    if (contentWidth <= 0 || contentHeight <= 0) return;

    const padding = 40;
    const scaleX = (this.dc.width - padding * 2) / contentWidth;
    const scaleY = (this.dc.height - padding * 2) / contentHeight;
    const zoom = Math.min(scaleX, scaleY, 5);

    this.dc.zoom = zoom;
    this.dc.scrollX = (this.dc.width / 2) - ((minX + maxX) / 2) * zoom;
    this.dc.scrollY = (this.dc.height / 2) - ((minY + maxY) / 2) * zoom;
    this.dc.markDirty();
    this.updateStatusZoom();
  }

  // Document management

  newDocument() {
    const id = generateId();
    const now = new Date().toISOString();
    this.engine.set_document_id(id);
    this.engine.set_document_name('untitled');
    this.engine.set_created_at(now);
    this.engine.clear_selection();
    this.engine.set_viewport(0, 0, 1);
    this.dc.markDirty();
    this.updateStatus();
  }

  async loadDrawing(id) {
    try {
      const doc = await API.loadDrawing(id);
      const json = JSON.stringify(doc);
      this.engine.load_document(json);
      if (doc.view) {
        this.engine.set_viewport(doc.view.scroll_x || 0, doc.view.scroll_y || 0, doc.view.zoom || 1);
      }
      this.dc.markDirty();
      this.updateStatus();
    } catch (e) {
      console.error('Failed to load drawing:', e);
      this.newDocument();
    }
  }

  buildDocument() {
    const docJson = this.engine.get_document_json_for_save();
    const doc = JSON.parse(docJson);
    // Include current viewport
    doc.view = {
      scroll_x: this.dc.scrollX,
      scroll_y: this.dc.scrollY,
      zoom: this.dc.zoom,
    };
    return doc;
  }

  async save() {
    const savedEl = document.getElementById('status-saved');
    try {
      const doc = this.buildDocument();
      await API.saveDrawing(doc);
      this.isDirty = false;
      this.updateStatusName();
      savedEl.textContent = 'Saved';
      savedEl.classList.remove('error');
      savedEl.classList.add('saved');
      setTimeout(() => {
        savedEl.textContent = '';
        savedEl.classList.remove('saved');
      }, 2000);
    } catch (e) {
      console.error('Failed to save:', e);
      savedEl.textContent = 'Save failed';
      savedEl.classList.remove('saved');
      savedEl.classList.add('error');
    }
  }

  async exportSvg() {
    const btn = document.getElementById('btn-export');
    btn.disabled = true;
    const savedEl = document.getElementById('status-saved');
    try {
      const doc = this.buildDocument();
      const svg = await API.exportSvg(doc);
      const blob = new Blob([svg], { type: 'image/svg+xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${this.engine.document_name()}.svg`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Failed to export:', e);
      savedEl.textContent = 'Export failed';
      savedEl.classList.add('error');
      setTimeout(() => { savedEl.textContent = ''; savedEl.classList.remove('error'); }, 3000);
    } finally {
      btn.disabled = false;
    }
  }

  async exportPng() {
    const btn = document.getElementById('btn-export-png');
    btn.disabled = true;
    const savedEl = document.getElementById('status-saved');
    try {
      const doc = this.buildDocument();
      const blob = await API.exportPng(doc);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${this.engine.document_name()}.png`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Failed to export PNG:', e);
      savedEl.textContent = 'Export failed';
      savedEl.classList.add('error');
      setTimeout(() => { savedEl.textContent = ''; savedEl.classList.remove('error'); }, 3000);
    } finally {
      btn.disabled = false;
    }
  }

  // Status bar

  updateStatus() {
    this.updateStatusName();
    this.updateStatusZoom();
    this._updateToolStatus();
    // Show/hide welcome hint based on element count
    const hint = document.getElementById('welcome-hint');
    if (hint) {
      hint.classList.toggle('hidden', this.engine.element_count() > 0);
    }
  }

  updateStatusName() {
    const el = document.getElementById('status-name');
    // Don't overwrite if inline rename is active
    if (el.querySelector('input')) return;
    const name = this.engine.document_name() + (this.isDirty ? ' *' : '');
    el.textContent = name;
  }

  _startInlineRename() {
    const el = document.getElementById('status-name');
    if (el.querySelector('input')) return; // already editing
    const current = this.engine.document_name();
    const input = document.createElement('input');
    input.type = 'text';
    input.value = current;
    input.className = 'status-rename-input';
    el.textContent = '';
    el.appendChild(input);
    input.focus();
    input.select();
    const commit = () => {
      const val = input.value.trim();
      if (val && val !== current) {
        this.engine.set_document_name(val);
        this.isDirty = true;
      }
      this.updateStatusName();
    };
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') { e.preventDefault(); commit(); }
      if (e.key === 'Escape') { e.preventDefault(); this.updateStatusName(); }
    });
    input.addEventListener('blur', commit);
  }

  updateStatusZoom() {
    const zoomText = Math.round(this.dc.zoom * 100) + '%';
    const zoomBtn = document.getElementById('zoom-btn');
    if (zoomBtn) zoomBtn.textContent = zoomText;
  }
}

function generateId() {
  return crypto.randomUUID();
}

// Boot
window.addEventListener('DOMContentLoaded', () => {
  App.create().then(app => { window.app = app; });
});
