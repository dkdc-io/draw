/* @ts-self-types="./draw_wasm.d.ts" */

/**
 * The main WASM-facing engine. Holds all state needed for rendering and
 * interaction: document, renderer, viewport, selection, and undo history.
 */
export class DrawEngine {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DrawEngineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_drawengine_free(ptr, 0);
    }
    /**
     * Add an element from JSON. Returns the element ID, or empty string on failure.
     * @param {string} json
     * @returns {string}
     */
    add_element(json) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            wasm.drawengine_add_element(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred2_0 = r0;
            deferred2_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Add an element to the selection.
     * @param {string} id
     */
    add_to_selection(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_add_to_selection(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {boolean}
     */
    can_redo() {
        const ret = wasm.drawengine_can_redo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    can_undo() {
        const ret = wasm.drawengine_can_undo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Clear the selection.
     */
    clear_selection() {
        wasm.drawengine_clear_selection(this.__wbg_ptr);
    }
    /**
     * Clear the rubber-band selection box.
     */
    clear_selection_box() {
        wasm.drawengine_clear_selection_box(this.__wbg_ptr);
    }
    /**
     * @returns {string}
     */
    document_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_document_id(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    document_name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_document_name(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    element_count() {
        const ret = wasm.drawengine_element_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get element IDs within a world-coordinate rectangle. Returns JSON array.
     * @param {number} x
     * @param {number} y
     * @param {number} w
     * @param {number} h
     * @returns {string}
     */
    elements_in_rect(x, y, w, h) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_elements_in_rect(retptr, this.__wbg_ptr, x, y, w, h);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get all element IDs as a JSON array.
     * @returns {string}
     */
    get_all_element_ids() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_get_all_element_ids(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get the full document JSON with updated modified_at for saving.
     * @returns {string}
     */
    get_document_json_for_save() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_get_document_json_for_save(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get an element as JSON, or empty string if not found.
     * @param {string} id
     * @returns {string}
     */
    get_element(id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            wasm.drawengine_get_element(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred2_0 = r0;
            deferred2_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get elements with a specific group_id (for bound text). Returns JSON array of IDs.
     * @param {string} group_id
     * @returns {string}
     */
    get_elements_by_group(group_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(group_id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            wasm.drawengine_get_elements_by_group(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred2_0 = r0;
            deferred2_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the current selection as a JSON array of strings.
     * @returns {string}
     */
    get_selection() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_get_selection(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get text overlay data for browser-native text rendering.
     * Returns JSON array of text elements with screen-space positions:
     * `[{"x":..,"y":..,"text":"..","fontSize":..,"fontFamily":"..","align":"..","color":"..","opacity":..,"width":..,"height":..}]`
     * @returns {string}
     */
    get_text_overlays() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_get_text_overlays(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Hit test at screen coordinates. Returns element ID or empty string.
     * @param {number} screen_x
     * @param {number} screen_y
     * @returns {string}
     */
    hit_test(screen_x, screen_y) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_hit_test(retptr, this.__wbg_ptr, screen_x, screen_y);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Hit test for resize handles. Returns JSON `{"id":"...","handle":"NorthWest"}`
     * or empty string if no handle hit.
     * @param {number} screen_x
     * @param {number} screen_y
     * @returns {string}
     */
    hit_test_handle(screen_x, screen_y) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_hit_test_handle(retptr, this.__wbg_ptr, screen_x, screen_y);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Check if an element is selected.
     * @param {string} id
     * @returns {boolean}
     */
    is_selected(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.drawengine_is_selected(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Load a document from a JSON string.
     * @param {string} json
     * @returns {boolean}
     */
    load_document(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.drawengine_load_document(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Move an element to absolute position (x, y).
     * @param {string} id
     * @param {number} x
     * @param {number} y
     */
    move_element(id, x, y) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_move_element(this.__wbg_ptr, ptr0, len0, x, y);
    }
    /**
     * Create a new engine with the given canvas dimensions and device pixel ratio.
     * @param {number} width
     * @param {number} height
     * @param {number} pixel_ratio
     */
    constructor(width, height, pixel_ratio) {
        const ret = wasm.drawengine_new(width, height, pixel_ratio);
        this.__wbg_ptr = ret >>> 0;
        DrawEngineFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Push a raw action for undo support. The action_json must match the Action
     * enum serialization format.
     * @param {string} action_json
     * @returns {boolean}
     */
    push_action(action_json) {
        const ptr0 = passStringToWasm0(action_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.drawengine_push_action(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Redo the last undone action. Returns true if something was redone.
     * @returns {boolean}
     */
    redo() {
        const ret = wasm.drawengine_redo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Remove an element by ID. Returns true if the element existed.
     * @param {string} id
     * @returns {boolean}
     */
    remove_element(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.drawengine_remove_element(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Remove multiple elements by ID (JSON array). Pushes a batch undo action.
     * @param {string} ids_json
     */
    remove_elements(ids_json) {
        const ptr0 = passStringToWasm0(ids_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_remove_elements(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Remove an element from the selection.
     * @param {string} id
     */
    remove_from_selection(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_remove_from_selection(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Render the current state and return RGBA pixel data.
     * @returns {Uint8Array}
     */
    render() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_render(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Return the height of the rendered pixmap in physical pixels.
     * @returns {number}
     */
    render_height() {
        const ret = wasm.drawengine_render_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return the width of the rendered pixmap in physical pixels.
     * @returns {number}
     */
    render_width() {
        const ret = wasm.drawengine_render_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Move an element one position backward in draw order.
     * @param {string} id
     */
    reorder_backward(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_reorder_backward(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Move an element one position forward in draw order.
     * @param {string} id
     */
    reorder_forward(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_reorder_forward(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Move an element to the back (bottom of draw order).
     * @param {string} id
     */
    reorder_to_back(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_reorder_to_back(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Move an element to the front (top of draw order).
     * @param {string} id
     */
    reorder_to_front(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_reorder_to_front(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Resize an element to the given bounds.
     * @param {string} id
     * @param {number} x
     * @param {number} y
     * @param {number} w
     * @param {number} h
     */
    resize_element(id, x, y, w, h) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_resize_element(this.__wbg_ptr, ptr0, len0, x, y, w, h);
    }
    /**
     * Serialize the current document to JSON.
     * @returns {string}
     */
    save_document() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_save_document(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Convert screen coordinates to world coordinates. Returns JSON `{"x":...,"y":...}`.
     * @param {number} sx
     * @param {number} sy
     * @returns {string}
     */
    screen_to_world(sx, sy) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.drawengine_screen_to_world(retptr, this.__wbg_ptr, sx, sy);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    scroll_x() {
        const ret = wasm.drawengine_scroll_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    scroll_y() {
        const ret = wasm.drawengine_scroll_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * Select all elements (skipping bound text — they follow parent shapes).
     */
    select_all() {
        wasm.drawengine_select_all(this.__wbg_ptr);
    }
    /**
     * @param {string} ts
     */
    set_created_at(ts) {
        const ptr0 = passStringToWasm0(ts, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_set_created_at(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string} id
     */
    set_document_id(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_set_document_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string} name
     */
    set_document_name(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_set_document_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Set the selected element IDs (JSON array of strings).
     * @param {string} ids_json
     */
    set_selection(ids_json) {
        const ptr0 = passStringToWasm0(ids_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.drawengine_set_selection(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Set the rubber-band selection box (screen coordinates).
     * @param {number} x
     * @param {number} y
     * @param {number} w
     * @param {number} h
     */
    set_selection_box(x, y, w, h) {
        wasm.drawengine_set_selection_box(this.__wbg_ptr, x, y, w, h);
    }
    /**
     * Show or hide the grid.
     * @param {boolean} show
     */
    set_show_grid(show) {
        wasm.drawengine_set_show_grid(this.__wbg_ptr, show);
    }
    /**
     * Update canvas dimensions (e.g. on window resize).
     * @param {number} width
     * @param {number} height
     */
    set_size(width, height) {
        wasm.drawengine_set_size(this.__wbg_ptr, width, height);
    }
    /**
     * Set the viewport (scroll and zoom).
     * @param {number} scroll_x
     * @param {number} scroll_y
     * @param {number} zoom
     */
    set_viewport(scroll_x, scroll_y, zoom) {
        wasm.drawengine_set_viewport(this.__wbg_ptr, scroll_x, scroll_y, zoom);
    }
    /**
     * Undo the last action. Returns true if something was undone.
     * @returns {boolean}
     */
    undo() {
        const ret = wasm.drawengine_undo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Update an element's style from JSON. The JSON should contain style fields
     * (stroke, fill, font, opacity, etc.) to merge into the element.
     * @param {string} id
     * @param {string} style_json
     * @returns {boolean}
     */
    update_element_style(id, style_json) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(style_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.drawengine_update_element_style(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    zoom() {
        const ret = wasm.drawengine_zoom(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) DrawEngine.prototype[Symbol.dispose] = DrawEngine.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_81fc77679af83bc6: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_getRandomValues_d49329ff89a07af1: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getTime_f6ac312467f7cf09: function(arg0) {
            const ret = getObject(arg0).getTime();
            return ret;
        },
        __wbg_new_0_bfa2ef4bc447daa2: function() {
            const ret = new Date();
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./draw_wasm_bg.js": import0,
    };
}

const DrawEngineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_drawengine_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('draw_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
