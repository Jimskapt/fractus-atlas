/** @format */

// used by wasm :
const { invoke } = window.__TAURI__.core;

var modified = false;

// used by wasm :
function set_modified() {
	modified = true;
}

window.onbeforeunload = function () {
	if (modified) {
		return 'unsaved changes';
	} else {
		return null;
	}
};
