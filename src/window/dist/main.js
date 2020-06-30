// this file will be included in /src/window/dist/main.html by /src/window/mod.rs:run

'use strict';

let STANDALONE_MODE = true; // used when testing HTML in external browser
setTimeout(function () {
	if (STANDALONE_MODE) {
		App.remote.debug = true;
		App.remote.receive.set_targets(['./target-1/', './target-2/', './target-3/']);
		App.methods.browse_folders(false);
		App.data.move_folders_history = ['folder-B', 'folder-D'];
		App.data.internal_server_port = 4040;
		App.remote.receive.set_images_count(1);
		App.remote.receive.set_active(1, 'a/b/c/d.jpg', 'token', false);

		App.remote.receive.set_move_folders(['folder-A', 'folder-B', 'folder-C', 'folder-D']);

		App.methods.refresh_image();
	}
}, 2000);

const ALPHABET = 'abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ';

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

function previous_move_folder() {
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
}

function next_move_folder() {
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
}

document.addEventListener("DOMContentLoaded", function () {

	setTimeout(function () {
		document.getElementById('image').addEventListener('error', App.methods.image_not_found_error);
	}, 1000);

	const menu_click = function (e, button) {
		if (button === undefined || button === null) {
			button = 'left';
		}

		const custom_menu = document.getElementById('custom_menu');
		if (custom_menu !== null && custom_menu !== undefined) {
			if (e.target.id === 'image' && button === 'right') {
				e.preventDefault();

				custom_menu.style.display = 'block';
				custom_menu.style.left = e.pageX + 'px';
				custom_menu.style.top = e.pageY + 'px';
			} else {
				custom_menu.style.display = 'none';
			}
		}
	};

	document.addEventListener('click', function (e) {
		menu_click(e, 'left');
	});
	document.addEventListener('contextmenu', function (e) {
		menu_click(e, 'right');
	});

	document.getElementById('move_search').addEventListener('keyup', function (event) {
		// console.log(event);

		if (App.data.mode === 'move') {
			if (event.keyCode === 38) { // UP
				previous_move_folder();
			} else if (event.keyCode === 40) { // DOWN
				next_move_folder();
			} else if (event.keyCode === 13) { // ENTER
				if (!document.getElementById('move_ok').disabled) {
					App.methods.do_move();
				}
			} else {
				App.methods.refresh_move_folders_results(event.target.value);
			}
		}

	});

	App.data.selected_folder.init();
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
