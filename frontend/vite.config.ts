import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";

export default {
  plugins: [tailwindcss(), react()],
  server: { proxy: { "/api": "http://localhost:8082" } },
};
