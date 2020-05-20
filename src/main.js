// this file will be included in main.html

'use strict';

let App = {
	data: {
		mode: 'browse',
		position: 0,
		current_image: "",
		images_count: 0,
		folders: [],
		selected_folder: {
			_value: "",
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
		}
	},
	methods: {
		refresh_image: function () {

			if (App.data.images_count > 0) {

				const nodes = document.querySelectorAll("#progress button");
				for (let i = 0; i < nodes.length; i++) {
					nodes[i].disabled = false;
				}
				document.getElementById("position_input").disabled = false;
				document.getElementById("main_next").disabled = false;
				document.getElementById("main_previous").disabled = false;
				document.getElementById("image").style.display = "block";
				document.getElementById("no-images-error").style.display = "none";

			} else if (App.data.images_count <= 0) {

				const nodes = document.querySelectorAll("#progress button");
				for (let i = 0; i < nodes.length; i++) {
					nodes[i].disabled = true;
				}
				document.getElementById("position_input").disabled = true;
				document.getElementById("main_next").disabled = true;
				document.getElementById("main_previous").disabled = true;
				document.getElementById("image").style.display = "none";
				document.getElementById("no-images-error").style.display = "block";

			}

			document.getElementById("image").src = "file:///" + App.data.current_image;
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
		do_move: function () {
			App.remote.send({
				instruction: 'Move',
				into: App.data.selected_folder.get()
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
			set_current: function (position, path) {
				App.data.position = position;
				App.data.current_image = path;
				App.methods.refresh_image();
			},
			preload: function (path) {
				let preloadLink = document.createElement("link");
				preloadLink.href = 'file:///' + path;
				preloadLink.rel = "preload";
				preloadLink.as = "image";
				document.head.appendChild(preloadLink);
			}
		},
		send: function (argument) {
			if (typeof argument === 'string') {
				argument = {
					instruction: argument,
				};
			}

			external.invoke(JSON.stringify(argument));
		}
	}
}

document.addEventListener("keyup", function (event) {
	// console.log(event);

	if (App.data.mode === 'move') {
		if (event.keyCode === 27) { // echap
			App.methods.toggle_move_window();
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
	App.methods.refresh_image();
	App.methods.refresh_folders_result("");

	document.getElementById('move_search').addEventListener("keyup", function (event) {
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
