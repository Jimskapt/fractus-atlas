/** @format */

// used by wasm :
const { invoke } = window.__TAURI__.core;

var modified = false;

function change_settings_path(event) {
	if (!modified) {
		invoke('set_settings_path', {
			settingsPath: event.target.value,
		});

		setTimeout(function () {
			location.reload(true);
		}, 500);
	} else {
		set_modified(true);
	}
}

window.addEventListener('DOMContentLoaded', function () {
	document
		.querySelector('#settings_path')
		.addEventListener('blur', change_settings_path);
});

window.onbeforeunload = function () {
	if (modified) {
		return 'unsaved changes';
	} else {
		return null;
	}
};

// used by wasm :
function set_modified(do_no_lock_save_path) {
	modified = true;

	const container = document.querySelector('#save_messages');
	container.childNodes.forEach(function (node) {
		container.removeChild(node);
	});

	const unsaved = document.createElement('p');
	unsaved.classList.add('warning');
	unsaved.innerText = '🚧 unsaved changes detected';

	container.appendChild(unsaved);

	if (!(do_no_lock_save_path === true)) {
		document
			.querySelector('#settings_path')
			.setAttribute('readonly', 'readonly');

		unsaved.innerText +=
			', settings save file path is now locked until saving or abort';
	}
}
function set_unmodified() {
	modified = false;

	document.querySelector('#settings_path').removeAttribute('readonly');
}
function is_modified() {
	return modified;
}
