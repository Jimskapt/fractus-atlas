/** @format */

import init, { build_settings_form, save_settings } from './wasm/ui.js';

async function run() {
	await init();
	await build_settings_form();

	document
		.querySelector('button#save')
		.addEventListener('click', function (event) {
			save_settings();
			event.preventDefault();
		});
}

run();
