import { BrowserRouter, Route, Routes } from "react-router-dom";
import PersonaListPage from "./pages/PersonaListPage";
import ChatPage from "./pages/ChatPage";
import CreatePersonaPage from "./pages/CreatePersonaPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<PersonaListPage />} />
        <Route path="/chat/:id" element={<ChatPage />} />
        <Route path="/create" element={<CreatePersonaPage />} />
      </Routes>
    </BrowserRouter>
  );
}
