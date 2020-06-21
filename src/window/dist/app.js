// this file will be included in /src/window/dist/main.html by /src/window/mod.rs:run

'use strict';

let App = {
	data: {
		debug: false,
		mode: 'browse',
		position: 0,
		active_image: '',
		active_already_moved: false,
		images_count: 0,
		move_folders: [],
		target_folders: [],
		internal_server_port: undefined,
		internal_server_token: '',
		selected_folder: {
			_value: '',
			init: function () {
				App.data.selected_folder.set('');
			},
			get: function () {
				return App.data.selected_folder._value;
			},
			set: function (new_value, propagate_break) {
				if (new_value !== undefined && new_value !== null) {
					App.data.selected_folder._value = new_value;
					if (propagate_break !== true) {
						App.methods.refresh_move_folders_results();
					}

					if (new_value.trim() !== '') {
						document.getElementById('move_ok').disabled = false;

						if (App.data.selected_folder.get() === '*move_back_to_origin*') {
							document.getElementById('confirmation-move').innerHTML = 'Current image will be <strong>moved back inside its origin folder</strong>.';
						} else {
							document.getElementById('confirmation-move').innerHTML = 'Current image will be moved inside the `' + App.data.selected_folder.get() + '/` sub-folder of the working folder.';
						}
					} else {
						document.getElementById('move_ok').disabled = true;
						document.getElementById('confirmation-move').innerHTML = '';
					}
				}
			}
		},
	},
	methods: {
		refresh_image: function () {

			if (App.data.images_count > 0) {

				document.getElementById("header_random").disabled = false;
				document.getElementById("header_move").disabled = false;

				document.getElementById("position_input").disabled = false;
				document.getElementById("main_next").disabled = false;
				document.getElementById("main_previous").disabled = false;
				document.getElementById("secondary_next").disabled = false;
				document.getElementById("secondary_previous").disabled = false;
				document.getElementById("image").style.display = "block";
				document.getElementById("no-images-error").style.display = "none";

			} else {

				document.getElementById("header_random").disabled = true;
				document.getElementById("header_move").disabled = true;

				document.getElementById("position_input").disabled = true;
				document.getElementById("main_next").disabled = true;
				document.getElementById("main_previous").disabled = true;
				document.getElementById("secondary_next").disabled = true;
				document.getElementById("secondary_previous").disabled = true;
				document.getElementById("image").style.display = "none";
				document.getElementById("no-images-error").style.display = "block";

			}

			const image = document.getElementById("image");
			image.alt = 'image';
			image.title = '';
			if (App.data.internal_server_port !== undefined) {
				image.src = 'http://127.0.0.1:' + App.data.internal_server_port + '/' + App.data.internal_server_token;
			} else {
				image.src = 'file:///' + App.data.active_image;
			}
			image.style.marginTop = '0';

			document.getElementById("imgpath").value = App.data.active_image;
			document.getElementById("max_counter").innerHTML = App.data.images_count;

			if (App.data.active_image === "") {
				document.getElementById("position_input").value = 0;
			} else {
				document.getElementById("position_input").value = App.data.position + 1;
			}

		},
		refresh_move_folders_results: function (search_value) {
			const move_search = document.getElementById('move_search');

			if (search_value === undefined || search_value === null) {
				search_value = move_search.value;
			}

			const sanitized = sanitize_folder_name(move_search.value.toLowerCase(), '-');
			if (move_search.value !== sanitized) {
				let selectionRange = {
					start: move_search.selectionStart,
					end: move_search.selectionEnd,
					direction: move_search.selectionDirection,
				};
				move_search.value = sanitized;
				move_search.setSelectionRange(selectionRange.start, selectionRange.end, selectionRange.direction);

				App.methods.refresh_move_folders_results(sanitized);
			} else {
				const move_search_results = document.getElementById('move_search_results');

				while (move_search_results.firstChild) {
					move_search_results.removeChild(move_search_results.lastChild);
				}

				let exact_match = false;
				const filtered_folders = App.data.move_folders.filter(function (folder) {
					if (folder === search_value) {
						exact_match = true;
					}

					return folder.includes(search_value) || search_value === '';
				});

				for (let i = 0; i < filtered_folders.length; i++) {
					const target_folder = filtered_folders[i];
					appendLabel(target_folder, target_folder);
				}

				if (!exact_match && search_value !== '') {
					appendLabel(search_value, '📂 move in new folder « ' + search_value + ' »', true);
				}

				if (App.data.active_already_moved) {
					appendLabel('*move_back_to_origin*', '⏪ move back to origin folder', true);
				}

				// need some time to render new DOM in browser before searching inside it ...
				setTimeout(function () {
					const search_if_one_checked = document.querySelector('input[type="radio"][name="selected_folder"]:checked');
					if (search_if_one_checked === null || search_if_one_checked === undefined) {
						const first_found = document.querySelector('input[type="radio"][name="selected_folder"]');
						if (first_found !== undefined && first_found !== null) {
							App.data.selected_folder.set(first_found.value);
						} else {
							App.data.selected_folder.set('', true);
						}
					}
				}, 200);
			}
		},
		toggle_move_window: function () {

			if (App.data.mode === 'move') {

				App.data.mode = 'browse';
				document.getElementById('move_search').blur();
				document.getElementById('move_window').style.display = 'none';

			} else if (App.data.images_count > 0) {

				App.methods.refresh_move_folders_results(document.getElementById('move_search').value);

				App.data.mode = 'move';

				document.getElementById('move_window').style.display = 'block';

				document.getElementById('move_search').focus();
				document.getElementById('move_search').setSelectionRange(0, document.getElementById('move_search').value.length);

			}
		},
		toggle_open_window: function () {

			if (App.data.mode === 'open') {

				App.data.mode = 'browse';
				document.getElementById('open_window').style.display = 'none';

			} else {

				App.data.mode = 'open';
				document.getElementById('open_window').style.display = 'block';

			}
		},
		do_move: function (toggle_popup_after) {
			if (toggle_popup_after !== true && toggle_popup_after !== false) {
				toggle_popup_after = true;
			}

			App.remote.send({
				instruction: 'DoMove',
				into: App.data.selected_folder.get(),
				toggle_popup: toggle_popup_after
			});
		},
		do_open: function (toggle_window) {
			if (toggle_window === undefined) {
				toggle_window = true;
			}

			App.remote.send({
				instruction: 'BrowseTargetFolders',
				folders: App.data.target_folders,
				toggle_window: toggle_window,
			});
		},
		request_position_change: function () {
			if (App.data.images_count > 0) {
				let value = Number(document.getElementById('position_input').value) - 1;
				if (value < 0) {
					value = 0;
				}

				if (Number.isInteger(value)) {
					App.remote.send({
						instruction: 'SetPosition',
						value: value
					});
				}
			}
		},
		image_not_found_error: function () {
			const image = document.getElementById('image');
			if (App.data.debug) {
				alert(image.src + ' has not been found');
			}
			image.alt = 'This image has not been found (has been moved ?)';
			image.title = 'This image has not been found (has been moved ?)';
			image.style.marginTop = '5em';
			image.src = BASE64_ERROR_IMAGE;

			ToastCenter.data.items.push('⁉ The file <strong>' + App.data.active_image + '</strong> was not found', -1, {
				classes: 'toast error'
			});
		}
	},
	remote: {
		receive: {
			set_images_count: function (value) {
				App.data.images_count = value;
				App.methods.refresh_image();
			},
			set_move_folders: function (value) {
				App.data.move_folders = value;
				App.methods.refresh_move_folders_results(document.getElementById('move_search').value);

				const menu_move_list = document.getElementById('menu_move_list');
				if (menu_move_list !== undefined && menu_move_list !== null) {
					while (menu_move_list.firstChild) {
						menu_move_list.removeChild(menu_move_list.lastChild);
					}

					const generateMenuItem = function (folder) {
						const li = document.createElement('li');
						li.textContent = folder;
						li.addEventListener('click', function () {
							App.data.selected_folder.set(folder);
							App.methods.do_move(false);
						});
						return li;
					};

					if (App.data.selected_folder.get() !== '' && App.data.selected_folder.get() !== '*move_back_to_origin*') {
						const li = generateMenuItem(App.data.selected_folder.get());
						li.className = 'active';
						menu_move_list.appendChild(li);
					}

					for (let i = 0; i < value.length; i++) {
						const li = generateMenuItem(value[i]);
						if (value[i] !== App.data.selected_folder.get()) {
							menu_move_list.appendChild(li);
						}
					}
				}
			},
			set_active: function (position, path, token, is_active_already_moved) {
				App.data.position = position;
				App.data.active_image = path;
				App.data.internal_server_token = token;
				App.data.active_already_moved = is_active_already_moved;
				App.methods.refresh_image();
			},
			preload: function (token) {
				(new Image()).src = 'http://127.0.0.1:' + App.data.internal_server_port + '/' + token;
			},
			set_targets: function (targets) {

				App.data.target_folders = targets;

				const inputs = document.getElementById('target_folders_inputs');

				while (inputs.firstChild) {
					inputs.removeChild(inputs.lastChild);
				}

				const browse_prefix = 'browse-';
				const click_listener = function (event) {
					App.remote.send({
						instruction: 'ShowBrowseTarget',
						id: Number(event.target.id.substring(browse_prefix.length))
					});
				};

				for (let i = 0; i < App.data.target_folders.length; i++) {
					const target = App.data.target_folders[i];

					const input = document.createElement('input');
					input.setAttribute('type', 'text');
					input.setAttribute('value', target);
					input.setAttribute('disabled', true);
					inputs.appendChild(input);

					const button = document.createElement('button');
					button.innerHTML = '📂 Browse ...';
					button.id = browse_prefix + i;
					button.addEventListener('click', click_listener);
					inputs.appendChild(button);

					inputs.appendChild(document.createElement('br'));
				}

				const input = document.createElement('input');
				input.setAttribute('type', 'text');
				input.setAttribute('value', '');
				input.setAttribute('disabled', true);
				inputs.appendChild(input);

				const button = document.createElement('button');
				button.innerHTML = '📂 Browse ...';
				button.addEventListener('click', function () {
					App.remote.send({
						instruction: 'ShowBrowseTarget',
						id: targets.length
					});
				});
				inputs.appendChild(button);

				inputs.appendChild(document.createElement('br'));
			}
		},
		send: function (argument) {
			if (!STANDALONE_MODE) {
				if (typeof argument === 'string') {
					argument = {
						instruction: argument,
					};
				}

				external.invoke(JSON.stringify(argument));
			} else {
				console.log(JSON.stringify(argument), 'has not been sent (STANDALONE_MODE != false)');
			}
		}
	}
};

const allowed_folder_chars = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";

function sanitize_folder_name(name, replacement) {
	let result = "";

	for (let i = 0; i < name.length; i++) {
		const char = name[i];
		if (allowed_folder_chars.indexOf(char) >= 0) {
			result += char;
		} else {
			result += replacement;
		}
	}

	return result;
}

function appendLabel(value, text_label, is_special) {
	if (is_special !== true) {
		is_special = false;
	}

	const label = document.createElement('label');
	if (is_special) {
		label.setAttribute('class', 'special');
	}

	const radio = document.createElement('input');
	radio.setAttribute('type', 'radio');
	radio.setAttribute('name', 'selected_folder');
	radio.setAttribute('value', value);
	if (App.data.selected_folder.get() === value) {
		radio.setAttribute('checked', true);
	}

	radio.addEventListener('click', function () {
		App.data.selected_folder.set(value);
		App.methods.do_move();
	});

	const labelText = document.createTextNode(text_label);

	label.appendChild(radio);
	label.appendChild(labelText);

	document.getElementById('move_search_results').appendChild(label);
}
