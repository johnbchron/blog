module.exports = {
	content: [
    "./crates/**/*.rs"
	],
	theme: {
		fontFamily: {
			'sans': ['FiraSans', 'ui-sans-serif', 'system-ui', 'sans-serif', "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"],
			'mono': ['Iosevka Custom', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', "Liberation Mono", "Courier New", 'monospace'],
		},
		extend: {
			colors: {
				'periwinkle': "oklch(68% 0.164 273.6)",
			},
		},
	},
}
