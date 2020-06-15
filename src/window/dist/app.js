// this file will be included in /src/window/dist/main.html by /src/window/mod.rs:run

let App = {
	data: {
		debug: false,
		mode: 'browse',
		position: 0,
		active_image: '',
		active_already_moved: false,
		images_count: 0,
		folders: [],
		target_folders: [],
		internal_server_port: undefined,
		internal_server_token: '',
		selected_folder: {
			_value: '',
			get: function () {
				return App.data.selected_folder._value;
			},
			set: function (new_value) {
				App.data.selected_folder._value = new_value;

				if (App.data.selected_folder.get() === "") {
					document.getElementById('confirmation-move').innerHTML = '';
					document.getElementById('move_ok').disabled = true;
				} else {
					if (App.data.selected_folder.get() === '*move_back_to_origin*') {
						document.getElementById('confirmation-move').innerHTML = 'Current image will be <strong>moved back inside its origin folder</strong>.';
					} else {
						document.getElementById('confirmation-move').innerHTML = 'Current image will be moved inside the `' + App.data.selected_folder.get() + '/` sub-folder of the working folder.';
					}
					document.getElementById('move_ok').disabled = false;
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
				document.getElementById("image").style.display = "block";
				document.getElementById("no-images-error").style.display = "none";

			} else {

				document.getElementById("header_random").disabled = true;
				document.getElementById("header_move").disabled = true;

				document.getElementById("position_input").disabled = true;
				document.getElementById("main_next").disabled = true;
				document.getElementById("main_previous").disabled = true;
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
		refresh_folders_result: function (value) {
			let found = 0;
			let search_items = "";
			let selected_is_inside = false;

			const filtered_folders = App.data.folders.filter(function (folder) {
				return value.trim() === "" || folder.trim().toLowerCase().includes(value.trim().toLowerCase());
			});

			for (let i = 0; i < filtered_folders.length; i++) {
				let folder = filtered_folders[i];

				search_items += '<label><input type="radio" name="selected_folder"';

				if (folder === App.data.selected_folder.get()) {
					search_items += ' checked';
					selected_is_inside = true;
				}

				search_items += ' value="' + folder + '" /> ' + folder + '</label><br>\n';
				found++;
			}

			if (!selected_is_inside && App.data.folders.length > 0) {
				App.data.selected_folder.set(filtered_folders[0]);
			}

			if (value.trim() !== "" && found <= 0) {
				search_items += '<label><input type="radio" name="selected_folder" value="' + value + '" checked /> ' + value +
					'</label><br>\n';
				found++;

				App.data.selected_folder.set(value);
			}

			if (App.data.active_already_moved) {
				search_items += '<label><input type="radio" name="selected_folder" value="*move_back_to_origin*" /> <i>⏪ Move back to origin folder</i></label><br>\n';
			}

			document.getElementById('move_search_results').innerHTML = search_items;

			if (App.data.selected_folder.get().trim() === '' || (!selected_is_inside && App.data.folders.length > 0)) {
				// TODO : improve-it !
				setTimeout(function () {
					const el = document.querySelector('input[name="selected_folder"]');

					if (el !== undefined && el !== null) {
						el.checked = true;
					}
				}, 300);
			}

			setTimeout(function () {
				const elements = document.querySelectorAll('input[name="selected_folder"]');

				for (let i = 0; i < elements.length; i++) {
					const el = elements[i];

					el.addEventListener('change', function () {
						App.data.selected_folder.set(el.value);
					});
				}
			}, 100);
		},
		toggle_move_window: function () {

			if (App.data.mode === 'move') {

				App.data.mode = 'browse';
				document.getElementById('move_search').blur();
				document.getElementById('move_window').style.display = 'none';

			} else if (App.data.images_count > 0) {

				App.methods.refresh_folders_result(document.getElementById('move_search').value);

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
		do_move: function () {
			App.remote.send({
				instruction: 'DoMove',
				into: App.data.selected_folder.get()
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
			set_folders: function (value) {
				App.data.folders = value;
				App.methods.refresh_folders_result(document.getElementById('move_search').value);
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
