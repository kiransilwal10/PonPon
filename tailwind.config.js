/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,vue,svelte}"
  ],
  theme: {
    extend: {
      colors: {
        // your custom pastel palette
        pinksoft: "#FDF1F4",
        mintsoft: "#E0F7FA",
        peachsoft: "#FFE4E1",
        accent:   "#FF9EB7",
        text:     "#4E4E4E",
      },
      borderRadius: {
        'xl': '1.25rem',  // 20px
        '2xl': '1.5rem',  // 24px
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
}
}
