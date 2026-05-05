import { NavLink, Outlet } from "react-router-dom";
import { Settings, Users } from "lucide-react";
import { OllamaStatusBadge } from "../components/OllamaStatusBadge";
import { useOllamaStatus } from "../hooks/useOllamaStatus";

const navLinkClass = ({ isActive }: { isActive: boolean }) =>
  `flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors ${
    isActive
      ? "bg-[#EDE5D8] font-medium text-[#2C2C2C]"
      : "text-[#6B6B6B] hover:bg-[#F0EBE3] hover:text-[#2C2C2C]"
  }`;

export default function AppLayout() {
  const ollamaStatus = useOllamaStatus();

  return (
    <div className="flex h-screen bg-[#FAF9F7] text-[#2C2C2C]">
      <aside className="flex w-60 flex-shrink-0 flex-col border-r border-[#E8E0D0] bg-[#FAF9F7]">
        {/* Logo */}
        <div className="flex h-14 items-center gap-2.5 px-4">
          <div className="grid h-7 w-7 place-items-center rounded-md bg-[#8B6F47] text-white">
            <span className="font-serif text-[15px] leading-none">A</span>
          </div>
          <span className="text-[17px] font-semibold tracking-tight">Animus</span>
        </div>

        {/* Nav */}
        <nav className="flex-1 px-2 py-2" aria-label="Main navigation">
          <NavLink to="/personas" end={false} className={navLinkClass}>
            <Users className="h-4 w-4 shrink-0" />
            Personas
          </NavLink>
        </nav>

        {/* Bottom */}
        <div className="space-y-1 border-t border-[#E8E0D0] px-2 py-3">
          <div className="px-3 py-1">
            <OllamaStatusBadge
              online={ollamaStatus.online}
              model={ollamaStatus.model}
            />
          </div>
          <NavLink to="/settings" className={navLinkClass}>
            <Settings className="h-4 w-4 shrink-0" />
            Settings
          </NavLink>
        </div>
      </aside>

      <main className="flex flex-1 flex-col overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
