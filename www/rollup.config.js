import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import rust from "@wasm-tool/rollup-plugin-rust";
import css from 'rollup-plugin-css-only';
import svelte from 'rollup-plugin-svelte';
import {terser} from 'rollup-plugin-terser';

// true when in production mode.
const production = !process.env.ROLLUP_WATCH;

export default {
	// Name of the main file.
	input: 'src/main.js',
	// Output parameters.
	output: {
		name: 'chess_engine_client',
		format: 'iife',
		sourcemap: true,
		file: 'public/build/bundle.js',
	},
	// Plugins used.
	plugins: [
		// Includes svelte files.
		svelte({
			compilerOptions: {
				// Enable run-time checks when not in production.
				dev: !production
			}
		}),
		// Single css file, better performance.
		css({
			output: 'bundle.css'
		}),
		// For the chess-wasm crate. Fetch the wasm module in the build directory.
		rust({
			serverPath: "/build/"
		}),
		// For npm external dependencies.
		resolve({
			browser: true,
			dedupe: ['svelte']
		}),
		commonjs(),
		// When in production, minify.
		production && terser(),
	],
	// Supress warnings from 3rd party code.
	onwarn: (warning, warn) => {
		if (warning.id.indexOf(__dirname + '/node_modules/') !== 0) warn(warning)
	},
};
