// this file will be included in main.html

let GLOBAL_MODE = 'browse';
let GLOBAL_POSITION = 0;
let GLOBAL_IMAGES = [];
let GLOBAL_FOLDERS = [];
let GLOBAL_SELECTED_FOLDER = {
	_value: "",
	get: function () { return this._value; },
	set: function (new_value) {
		this._value = new_value;

		if (this.get() === "") {
			document.getElementById('confirmation-move').innerHTML = '';
		} else {
			document.getElementById('confirmation-move').innerHTML = 'Current image will be moved inside the `' + this.get() + '/` sub-folder of the working folder.';
		}
	}
};

function invoke(payload) {
	const data = JSON.stringify(payload);
	external.invoke(JSON.stringify({
		instruction: "Receiving",
		data: data
	}));
	external.invoke(data);
}

function set_images(value) {

	GLOBAL_IMAGES = value;

	if (GLOBAL_POSITION >= GLOBAL_IMAGES.length) {
		GLOBAL_POSITION = 0;
	}

	refresh_image();

}

function new_current(value) {
	GLOBAL_IMAGES[GLOBAL_POSITION] = value;
}

function set_folders(value) {

	GLOBAL_FOLDERS = value;
	refresh_folders_result(document.getElementById('move_search').value);

}

function refresh_image() {

	if (GLOBAL_IMAGES.length > 0) {

		let nodes = document.querySelectorAll("#progress button");
		for (let i = 0; i < nodes.length; i++) {
			nodes[i].disabled = false;
		}
		document.getElementById("position_input").disabled = false;
		document.getElementById("main_next").disabled = false;
		document.getElementById("main_previous").disabled = false;
		document.getElementById("image").style.display = "block";
		document.getElementById("no-images-error").style.display = "none";

		document.getElementById("image").src = "file:///" + GLOBAL_IMAGES[GLOBAL_POSITION];
		document.getElementById("imgpath").value = GLOBAL_IMAGES[GLOBAL_POSITION];
		document.getElementById("position_input").value = GLOBAL_POSITION + 1;
		document.getElementById("max_counter").innerHTML = GLOBAL_IMAGES.length;

	} else if (GLOBAL_IMAGES.length <= 0) {

		let nodes = document.querySelectorAll("#progress button");
		for (let i = 0; i < nodes.length; i++) {
			nodes[i].disabled = true;
		}
		document.getElementById("position_input").disabled = true;
		document.getElementById("main_next").disabled = true;
		document.getElementById("main_previous").disabled = true;
		document.getElementById("image").style.display = "none";
		document.getElementById("no-images-error").style.display = "block";

		document.getElementById("image").src = "";
		document.getElementById("imgpath").value = "";
		document.getElementById("position_input").value = 0;
		document.getElementById("max_counter").innerHTML = 0;

	}

}

function previous() {
	invoke({
		instruction: 'Previous'
	});
}

function next() {
	invoke({
		instruction: 'Next'
	});
}

function random() {
	invoke({
		instruction: 'Random'
	});
}

function change_position() {
	let value = Number(document.getElementById("position_input").value);
	if (value <= 0) {
		value = 0;
	} else {
		value -= 1;
	}

	invoke({
		instruction: 'SetPosition',
		value: value
	});
}

function toggle_move_window() {

	if (GLOBAL_MODE === 'move') {
		GLOBAL_MODE = 'browse';

		document.getElementById('move_window').style.display = 'none';

		document.getElementById('move_search').blur();

	} else if (GLOBAL_IMAGES.length > 0) {
		GLOBAL_MODE = 'move';

		document.getElementById('move_window').style.display = 'block';

		document.getElementById('move_search').focus();
		document.getElementById('move_search').setSelectionRange(0, document.getElementById('move_search').value.length);

		document.getElementById('move_search').focus();

	}
}

function move() {
	if (GLOBAL_SELECTED_FOLDER.get().trim() !== "") {
		invoke({
			instruction: 'Move',
			into: GLOBAL_SELECTED_FOLDER.get()
		});
	}
}

function set_position(value) {
	GLOBAL_POSITION = value;

	refresh_image();
}

document.addEventListener("keyup", function (event) {
	// console.log(event);

	if (GLOBAL_MODE === 'move') {
		if (event.keyCode === 27) { // echap
			toggle_move_window();
		}
	} else {
		if (event.keyCode === 39) { // right
			next();
		} else if (event.keyCode === 37) { // left
			previous();
		} else if (event.key.toLowerCase() === "z") {
			random();
		} else if (event.key.toLowerCase() === "m") {
			toggle_move_window();
		}
	}
});

function refresh_folders_result(value) {
	let found = 0;
	let search_items = "";
	let selected_is_inside = false;

	for (let i = 0; i < GLOBAL_FOLDERS.length; i++) {
		let folder = GLOBAL_FOLDERS[i];

		if (value.trim() === "" || folder.trim().toLowerCase().includes(value.trim().toLowerCase())) {
			search_items += '<label><input type="radio" name="selected_folder"';

			if (folder === GLOBAL_SELECTED_FOLDER.get()) {
				search_items += " checked";
				selected_is_inside = true;
			}

			search_items += ' value="' + folder + '" /> ' + folder + '</label><br>';
			found++;
		}
	}

	if (!selected_is_inside && GLOBAL_FOLDERS.length > 0) {
		GLOBAL_SELECTED_FOLDER.set(GLOBAL_FOLDERS[0]);
	}

	if (value.trim() !== "" && found <= 0) {
		search_items += '<label><input type="radio" name="selected_folder" value="' + value + '" /> ' + value +
			'</label><br>';
		found++;

		GLOBAL_SELECTED_FOLDER.set(value);
	}

	if (found <= 0) {
		document.getElementById('move_ok').disabled = true;
	} else {
		document.getElementById('move_ok').disabled = false;
	}

	document.getElementById('move_search_results').innerHTML = search_items;

	setTimeout(function () {
		let el = document.querySelector('input[name="selected_folder"]');

		if (el !== undefined && el !== null) {
			el.checked = true;
		}
	}, 200);

	setTimeout(function () {
		let elements = document.querySelectorAll('input[name="selected_folder"]');

		for (let i = 0; i < elements.length; i++) {
			let el = elements[i];

			el.addEventListener('change', function () {
				GLOBAL_SELECTED_FOLDER.set(el.value);
			});
		}
	}, 100);
}

refresh_folders_result("");

document.getElementById('move_search').addEventListener("keyup", function (event) {
	// console.log(event);

	if (event.keyCode === 38) { // UP
		let nodes = document.querySelectorAll('input[name="selected_folder"]');
		let activate_previous = false;
		for (let i = nodes.length - 1; i >= 0; i--) {
			let node = nodes[i];

			if (node.checked && i > 0) {
				node.checked = false;
				activate_previous = true;

				continue;
			}

			if (activate_previous) {
				node.checked = true;
				activate_next = false;
				GLOBAL_SELECTED_FOLDER.set(node.value);
				break;
			}
		}
	} else if (event.keyCode === 40) { // DOWN
		let nodes = document.querySelectorAll('input[name="selected_folder"]');
		let activate_next = false;
		for (let i = 0; i < nodes.length; i++) {
			let node = nodes[i];

			if (node.checked && i < (nodes.length - 1)) {
				node.checked = false;
				activate_next = true;

				continue;
			}

			if (activate_next) {
				node.checked = true;
				activate_next = false;
				GLOBAL_SELECTED_FOLDER.set(node.value);
				break;
			}
		}
	} else if (event.keyCode === 13) { // ENTER
		move();
	} else {
		refresh_folders_result(event.target.value);
	}

});

document.addEventListener("DOMContentLoaded", function () {
	refresh_image();
});

// String.includes(...) polyfill :

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
