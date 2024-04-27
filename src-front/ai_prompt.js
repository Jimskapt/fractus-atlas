/** @format */

const { invoke } = window.__TAURI__.core;

window.addEventListener('DOMContentLoaded', async function (event) {
	let current_path = await invoke('get_current_path');
	let prompt = await invoke('get_ai_prompt');

	if (prompt != null && prompt != undefined) {
		this.document.querySelector('#ai_prompt').innerText =
			current_path + '\n\n' + prompt;
	} else {
		this.document.querySelector('#ai_prompt').innerText =
			current_path + '\n\n' + 'No prompt, or error.';
	}
});
