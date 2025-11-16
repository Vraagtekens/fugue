import "@lynx-js/preact-devtools";
import "@lynx-js/react/debug";
import { root } from "@lynx-js/react";
import { App } from "./App.jsx";

// root.render(<App />);

// if (import.meta.webpackHot) {
// 	import.meta.webpackHot.accept();
// }

// import "@lynx-js/preact-devtools";
// import "@lynx-js/react/debug";
// import { root } from "@lynx-js/react";
// import { MemoryRouter, Routes, Route } from "react-router";

// import { App } from "./App.jsx";
// import { Home } from "./routes/index.jsx";

// root.render(
// 	<MemoryRouter>
// 		<Routes>
// 			<Route path="/" element={<App />} />
// 			<Route path="/home" element={<Home />} />
// 		</Routes>
// 	</MemoryRouter>,
// );

// if (import.meta.webpackHot) {
// 	import.meta.webpackHot.accept();
// }

import { MemoryRouter, Route, Routes } from "react-router";
import { Home } from "./routes/home/index.js";

root.render(
	<MemoryRouter>
		<Routes>
			<Route path="/" element={<App />} />
			<Route path="/home" element={<Home />} />
		</Routes>
	</MemoryRouter>,
);

if (import.meta.webpackHot) {
	import.meta.webpackHot.accept();
}
