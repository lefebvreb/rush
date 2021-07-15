import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import rust from "@wasm-tool/rollup-plugin-rust";
import css from 'rollup-plugin-css-only';
import livereload from 'rollup-plugin-livereload';
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
				// enable run-time checks when not in production.
				dev: !production
			}
		}),
		// Single css file, better performance.
		css({ output: 'bundle.css' }),
		// For the chess-wasm crate.
		rust(),
		// For npm external dependencies.
		resolve({
			browser: true,
			dedupe: ['svelte']
		}),
		commonjs(),
		// When in dev mode, watch src and live reload.
		!production && livereload('public'),
		// When in production, minify.
		production && terser(),
	],
};
