// this file will be included in main.html by main.rs

'use strict';

let STANDALONE_MODE = true; // used when testing HTML in external browser
setTimeout(function () {
	if (STANDALONE_MODE) {
		App.remote.debug = true;
		App.remote.receive.set_targets(['./target-1/', './target-2/', './target-3/']);
		App.methods.do_open(false);
		App.remote.receive.set_folders(['./folder-A/', './folder-B/', './folder-C/', './folder-D/']);

		App.methods.refresh_image();
	}
}, 1000);

let App = {
	data: {
		debug: false,
		mode: 'browse',
		position: 0,
		current_image: '',
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
					document.getElementById('confirmation-move').innerHTML = 'Current image will be moved inside the `' + App.data.selected_folder.get() + '/` sub-folder of the working folder.';
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
				image.src = 'file:///' + App.data.current_image;
			}
			image.style.marginTop = '0';

			document.getElementById("imgpath").value = App.data.current_image;
			document.getElementById("max_counter").innerHTML = App.data.images_count;

			if (App.data.current_image === "") {
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
			set_current: function (position, path, token) {
				App.data.position = position;
				App.data.current_image = path;
				App.data.internal_server_token = token;
				App.methods.refresh_image();
			},
			preload: function (path) {
				let preloadLink = document.createElement("link");
				// TODO : internal_server
				preloadLink.href = 'file:///' + path;
				preloadLink.rel = "preload";
				preloadLink.as = "image";
				document.head.appendChild(preloadLink);
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

document.addEventListener("keyup", function (event) {
	// console.log(event);

	if (App.data.mode === 'move') {
		if (event.keyCode === 27) { // echap
			App.methods.toggle_move_window();
		}
	} else if (App.data.mode === 'open') {
		if (event.keyCode === 27) { // echap
			App.methods.toggle_open_window();
		}
	} else {
		if (event.keyCode === 39) { // right
			App.remote.send('Next');
		} else if (event.keyCode === 37) { // left
			App.remote.send('Previous');
		} else if (event.key.toLowerCase() === "z") {
			App.remote.send('Random');
		} else if (event.key.toLowerCase() === "m") {
			App.methods.toggle_move_window();
		}
	}
});

document.addEventListener("DOMContentLoaded", function () {

	setTimeout(function () {
		document.getElementById('image').addEventListener('error', App.methods.image_not_found_error);
	}, 1000);

	document.getElementById('move_search').addEventListener('keyup', function (event) {
		// console.log(event);

		if (App.data.mode === 'move') {
			if (event.keyCode === 38) { // UP
				const nodes = document.querySelectorAll('input[name="selected_folder"]');
				let activate_previous = false;
				for (let i = nodes.length - 1; i >= 0; i--) {
					const node = nodes[i];

					if (node.checked && i > 0) {
						node.checked = false;
						activate_previous = true;

						continue;
					}

					if (activate_previous) {
						node.checked = true;
						activate_previous = false;
						App.data.selected_folder.set(node.value);
						break;
					}
				}
			} else if (event.keyCode === 40) { // DOWN
				const nodes = document.querySelectorAll('input[name="selected_folder"]');
				let activate_next = false;
				for (let i = 0; i < nodes.length; i++) {
					const node = nodes[i];

					if (node.checked && i < (nodes.length - 1)) {
						node.checked = false;
						activate_next = true;

						continue;
					}

					if (activate_next) {
						node.checked = true;
						activate_next = false;
						App.data.selected_folder.set(node.value);
						break;
					}
				}
			} else if (event.keyCode === 13) { // ENTER
				App.methods.do_move();
			} else {
				App.methods.refresh_folders_result(event.target.value);
			}
		}

	});
});

// polyfills :

if (!String.prototype.includes) {
	String.prototype.includes = function (search, start) {
		'use strict';

		if (search instanceof RegExp) {
			throw TypeError('first argument must not be a RegExp');
		}
		if (start === undefined) {
			start = 0;
		}
		return this.indexOf(search, start) !== -1;
	};
}

Number.isInteger = Number.isInteger || function (value) {
	return typeof value === 'number' &&
		isFinite(value) &&
		Math.floor(value) === value;
};

if (!Array.prototype.filter) {
	Array.prototype.filter = function (func, thisArg) {
		'use strict';
		if (!((typeof func === 'Function' || typeof func === 'function') && this))
			throw new TypeError();

		var len = this.length >>> 0,
			res = new Array(len), // preallocate array
			t = this,
			c = 0,
			i = -1;

		var kValue;
		if (thisArg === undefined) {
			while (++i !== len) {
				// checks to see if the key was set
				if (i in this) {
					kValue = t[i]; // in case t is changed in callback
					if (func(t[i], i, t)) {
						res[c++] = kValue;
					}
				}
			}
		} else {
			while (++i !== len) {
				// checks to see if the key was set
				if (i in this) {
					kValue = t[i];
					if (func.call(thisArg, t[i], i, t)) {
						res[c++] = kValue;
					}
				}
			}
		}

		res.length = c; // shrink down array to proper size
		return res;
	};
}

const BASE64_ERROR_IMAGE = "data:image/png;base64,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";