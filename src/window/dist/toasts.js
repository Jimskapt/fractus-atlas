// this file will be included in /src/window/dist/main.html by /src/window/mod.rs:run

'use strict';

let ToastCenter = {
	data: {
		items: {
			_value: {},
			push: function (message, duration, options) {
				let id = '';
				const size = parseInt(Math.random() * 32);
				for (let i = 0; i < size; i++) {
					id += ALPHABET[parseInt(Math.random() * ALPHABET.length)];
				}

				let callback = undefined;
				duration = duration || 8000;
				if (Number.isInteger(duration) && duration > 0) {
					callback = setTimeout(function () {
						ToastCenter.data.items.remove(id);
					}, duration);
				}

				if (typeof (options) != "object") {
					options = {};
				}

				ToastCenter.data.items._value[id] = {
					message: message || '',
					style: options.style || '',
					classes: options.classes || options.class || ['toast info'],
					duration: duration,
					callback: callback
				};

				ToastCenter.methods.refresh_toasts();

				return id;
			},
			remove: function (id, force) {
				if (force !== true && force !== false) {
					force = false;
				}

				if (ToastCenter.data.items._value[id] !== undefined && (document.querySelector('#toast-' + id + ':hover') === null || force)) {
					delete ToastCenter.data.items._value[id];
					ToastCenter.methods.refresh_toasts();
				}
			}
		}
	},
	methods: {
		refresh_toasts: function () {
			const toasts = document.querySelectorAll('.toast');

			for (let i = 0; i < toasts.length; i++) {
				document.body.removeChild(toasts[i]);
			}

			const marginY = 10;
			let yPos = marginY;
			const keys = Object.keys(ToastCenter.data.items._value);
			for (let i = 0; i < keys.length; i++) {
				const id = keys[i];

				if (document.getElementById('toast-' + id) === null) {
					const toast = ToastCenter.data.items._value[id];

					const container = document.createElement('div');
					container.setAttribute('style', toast.style);
					container.setAttribute('class', toast.classes);
					container.setAttribute('id', 'toast-' + id);

					container.innerHTML += toast.message;

					container.addEventListener('click', function () {
						ToastCenter.data.items.remove(id, true);
					});

					document.body.appendChild(container);

					document.getElementById('toast-' + id).style.bottom = yPos + 'px';

					yPos += container.clientHeight + marginY;
				}
			}
		}
	}
};
