const cacheName = "v360_mips_emulator_pwa";
const filesToCache = [
	"./",
	"./index.html",
	"./mips_emulator_gui.js",
	"./mips_emulator_gui_bg.wasm",
];

// Start the service worker and cache all of the app's content
self.addEventListener('install', e => {
	e.waitUntil(
		caches.open(cacheName).then(function (cache) {
			return cache.addAll(filesToCache);
		})
	);
});

// Serve cached content when offline
self.addEventListener('fetch', e => {
	e.respondWith(
		caches.match(e.request).then(function (response) {
			return response || fetch(e.request);
		})
	);
});
