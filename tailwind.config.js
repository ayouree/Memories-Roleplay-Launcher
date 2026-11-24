module.exports = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: { extend: {
    fontFamily: { sans: ['Inter', 'Manrope', 'system-ui', 'sans-serif'] },
    colors: { bg: '#111111', bg2: '#181818', card: '#222222', elev: '#292929', input: '#191919',
      line: '#363636', txt2: '#A0A0A0', muted: '#707070', accent: '#D9D9D9',
      ok: '#65B889', warn: '#D6AA65', danger: '#E05252' } } },
  plugins: []
}
