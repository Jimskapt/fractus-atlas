/** @format */

const { invoke } = window.__TAURI__.core;

async function refresh() {
	document.querySelector('#preview_path').value = await invoke(
		'get_current_path'
	);
	document.querySelector('#preview').src =
		'http://image.localhost/?rand=' + Math.random() * 9999999;
}

async function change_path(event) {
	let changed = await invoke('change_path', {
		newPath: event.target.value,
	});

	if (changed) {
		refresh().await;
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
	let changed = await invoke('keyup', { key: event.key });

	if (changed) {
		refresh().await;
	}
});

window.addEventListener('DOMContentLoaded', async function (event) {
	refresh().await;

	const move_bar = document.querySelector('#move_bar');

	let move_button = document.createElement('button');
	move_button.innerText = '↩️ restore';
	move_button.addEventListener('click', async function () {
		if (await invoke('do_move', { name: '' })) {
			refresh().await;
		}
	});

	move_bar.appendChild(move_button);

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

	document.querySelector('#preview_path').addEventListener('blur', change_path);
	document
		.querySelector('#previous')
		.addEventListener('click', async function () {
			await change_position(-1);
		});
	document.querySelector('#next').addEventListener('click', async function () {
		await change_position(+1);
	});
});
