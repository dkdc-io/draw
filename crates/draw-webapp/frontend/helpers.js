// Shared helpers used by app.js and interactions.js

/** Parse an element from the WASM engine by ID. Returns null if not found. */
function getElement(engine, id) {
  const json = engine.get_element(id);
  if (!json) return null;
  try { return JSON.parse(json); } catch { return null; }
}

/** Get the current selection as an array of element IDs. */
function getSelection(engine) {
  try { return JSON.parse(engine.get_selection()); } catch { return []; }
}

/** Iterate over all bound text elements for each ID in `ids`, calling `callback(boundTextId, boundTextElement)`. */
function forEachBoundText(engine, ids, callback) {
  for (const id of ids) {
    let btIds;
    try { btIds = JSON.parse(engine.get_elements_by_group(id)); } catch { continue; }
    for (const btId of btIds) {
      const btEl = getElement(engine, btId);
      if (btEl) callback(btId, btEl);
    }
  }
}
