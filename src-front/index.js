/** @format */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

async function refresh() {
	document.querySelector('#preview').src =
		'http://image.localhost/?rand=' + Math.random() * 9999999;
	document.querySelector('#current-path').value = await invoke(
		'get_current_path'
	);

	var position = await invoke('get_current_position');
	if (position === null || position === undefined) {
		position = '?';

		document.querySelector('#open-folder').setAttribute('disabled', 'disabled');
		document.querySelector('#open-file').setAttribute('disabled', 'disabled');
		document
			.querySelector('#current-path')
			.setAttribute('disabled', 'disabled');
	} else {
		position += 1;

		document.querySelector('#open-folder').removeAttribute('disabled');
		document.querySelector('#open-file').removeAttribute('disabled');
		document.querySelector('#current-path').removeAttribute('disabled');
	}

	if (await invoke('current_can_be_restored')) {
		document.querySelector('#restore-origin').removeAttribute('disabled');
	} else {
		document
			.querySelector('#restore-origin')
			.setAttribute('disabled', 'disabled');
	}

	let display_position = position;
	if (position < 10) {
		display_position = '0' + display_position;
	}
	if (position < 100) {
		display_position = '0' + display_position;
	}

	document.querySelector('#position_counter').innerText =
		display_position + ' / ' + (await invoke('get_images_length'));

	let ai_prompt = await invoke('get_ai_prompt');
	if (ai_prompt != null && ai_prompt != undefined) {
		document.querySelector('#ai_prompt #ai_content').innerText =
			'🧠 A.I. prompt detected :\n\n' + ai_prompt;
		document.querySelector('#ai_prompt #ai_icon').removeAttribute('disabled');
	} else {
		document.querySelector('#ai_prompt #ai_content').innerText = '';
		document
			.querySelector('#ai_prompt #ai_icon')
			.setAttribute('disabled', 'disabled');
	}
}

async function change_path(event) {
	var rename = false;

	let current_path = await invoke('get_current_path');
	let new_path = event.target.value;

	if (current_path != new_path) {
		if (await invoke('is_confirm_rename')) {
			rename = confirm(
				`Would you want to rename ?\n${current_path}\ninto\n${new_path}`
			);
		} else {
			rename = true;
		}

		if (rename) {
			let changed = await invoke('change_path', {
				newPath: event.target.value,
			});

			if (changed) {
				refresh().await;
			}
		}
	}
}

async function change_position(step) {
	let changed = await invoke('change_position', {
		step,
	});

	if (changed) {
		refresh().await;
	}
}

window.addEventListener('keyup', async function (event) {
	if (document.activeElement !== this.document.querySelector('#current-path')) {
		let changed = await invoke('keyup', { key: event.key });

		if (changed) {
			refresh().await;
		}
	}
});

window.addEventListener('DOMContentLoaded', async function (event) {
	refresh().await;

	const move_bar = document.querySelector('#move-bar');

	let restore_button = document.querySelector('#restore-origin');
	restore_button.addEventListener('click', async function () {
		if (await invoke('do_move', { name: '' })) {
			refresh().await;
		}
	});

	const move_actions = await invoke('get_move_actions');
	for (let i = 0; i < move_actions.length; i++) {
		const move_action = move_actions[i];

		let move_button = document.createElement('button');
		move_button.innerText = move_action.name;
		move_button.addEventListener('click', async function () {
			if (await invoke('do_move', { name: move_action.name })) {
				refresh().await;
			}
		});

		move_bar.appendChild(move_button);
	}

	document.querySelector('#current-path').addEventListener('blur', change_path);

	document
		.querySelector('#previous')
		.addEventListener('click', async function () {
			await change_position(-1);
		});
	document
		.querySelector('#go-previous')
		.addEventListener('click', async function () {
			await change_position(-1);
		});

	document.querySelector('#next').addEventListener('click', async function () {
		await change_position(+1);
	});
	document
		.querySelector('#go-next')
		.addEventListener('click', async function () {
			await change_position(+1);
		});

	document
		.querySelector('#go-random')
		.addEventListener('click', async function () {
			let changed = await invoke('set_random_position');

			if (changed) {
				refresh().await;
			}
		});

	document
		.querySelector('#open-folder')
		.addEventListener('click', async function () {
			await invoke('os_open', { openTarget: 'folder' });
		});
	document
		.querySelector('#open-file')
		.addEventListener('click', async function () {
			await invoke('os_open', { openTarget: 'file' });
		});

	await invoke('update_files_list');
	this.setTimeout(refresh, 500);

	listen('request-ui-refresh', async function () {
		await refresh();
	});
});
