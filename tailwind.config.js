module.exports = {
	content: [
    "./app/src/**/*.rs"
	],
	theme: {
		fontFamily: {
			'sans': ['FiraSans', 'ui-sans-serif', 'system-ui', 'sans-serif', "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"],
			'mono': ['Iosevka Term Web', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', "Liberation Mono", "Courier New", 'monospace'],
		},
	},
	plugins: [ require("@tailwindcss/typography") ],
}
