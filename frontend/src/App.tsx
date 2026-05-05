import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import PersonaListPage from "./pages/PersonaListPage";
import ChatPage from "./pages/ChatPage";
import CreatePersonaPage from "./pages/CreatePersonaPage";
import EditPersonaPage from "./pages/EditPersonaPage";
import SettingsPage from "./pages/SettingsPage";
import AppLayout from "./layouts/AppLayout";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/personas" element={<PersonaListPage />} />
          <Route path="/personas/new" element={<CreatePersonaPage />} />
          <Route path="/personas/:id/edit" element={<EditPersonaPage />} />
          <Route path="/chat/:id" element={<ChatPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/" element={<Navigate to="/personas" />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
