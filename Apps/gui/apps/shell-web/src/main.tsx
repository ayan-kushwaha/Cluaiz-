import React from "react";
import { createRoot } from "react-dom/client";
import { AppShell, PanelFrame, ResizablePanel } from "@cluaiz/ui-core";
import { applyThemeTokensToDocument, createLayoutStore, createThemeStore } from "@cluaiz/app-core";
import type { AppMode } from "@cluaiz/protocol";
import "./globals.css";

const themeStore = createThemeStore();
const layoutStore = createLayoutStore();

type Workspace = { id: string; name: string; project: string; active?: boolean };
type ChatItem = { id: string; title: string; time: string; paused?: boolean };

const workspaces: Workspace[] = [
  { id: "w1", name: "Cluaiz Core", project: "Universal App", active: true },
  { id: "w2", name: "Inbox AI", project: "Conversation Ops" },
  { id: "w3", name: "Workflows", project: "Agent Automations" }
];

const pinnedChats: ChatItem[] = [
  { id: "c1", title: "Launch roadmap discussion", time: "2m" },
  { id: "c2", title: "Dashboard parity review", time: "12m", paused: true }
];

const todayChats: ChatItem[] = [
  { id: "c3", title: "WhatsApp style inbox UX", time: "21m" },
  { id: "c4", title: "Agent memory tuning", time: "46m" },
  { id: "c5", title: "Billing + settings sync", time: "1h" }
];

function Sidebar({ mode, onMode, onToggleRight, rightOpen }: {
  mode: AppMode;
  onMode: (m: AppMode) => void;
  onToggleRight: () => void;
  rightOpen: boolean;
}): JSX.Element {
  const [collapsed, setCollapsed] = React.useState(false);
  const [search, setSearch] = React.useState("");

  const filterChats = (items: ChatItem[]) =>
    items.filter((c) => c.title.toLowerCase().includes(search.toLowerCase()));

  return (
    <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
      <div className="sidebar-top">
        <div className="brand-row">
          <div className="brand-dot" />
          {!collapsed && <div className="brand-text"><strong>Cluaiz</strong><span>AI Workspace</span></div>}
          <button className="icon-btn" onClick={() => setCollapsed((v) => !v)}>{collapsed ? "»" : "«"}</button>
        </div>

        {!collapsed && (
          <div className="search-wrap">
            <input
              className="input"
              placeholder="Search chats, workspace..."
              value={search}
              onChange={(e) => setSearch(e.currentTarget.value)}
            />
          </div>
        )}
      </div>

      {!collapsed && (
        <div className="sidebar-scroll">
          <section className="sb-section">
            <div className="sb-title">Modes</div>
            <div className="mode-grid">
              <button className={`chip ${mode === "normal" ? "active" : ""}`} onClick={() => onMode("normal")}>Normal</button>
              <button className={`chip ${mode === "coding" ? "active" : ""}`} onClick={() => onMode("coding")}>Coding</button>
              <button className={`chip ${mode === "mobile_focus" ? "active" : ""}`} onClick={() => onMode("mobile_focus")}>Mobile</button>
            </div>
            <button className="chip full" onClick={onToggleRight}>{rightOpen ? "Hide Right Panel" : "Open Right Panel"}</button>
          </section>

          <section className="sb-section">
            <div className="sb-title">Workspace / Projects</div>
            <div className="list">
              {workspaces.map((w) => (
                <button key={w.id} className={`list-item ${w.active ? "active" : ""}`}>
                  <div>
                    <div className="li-title">{w.name}</div>
                    <div className="li-sub">{w.project}</div>
                  </div>
                </button>
              ))}
            </div>
          </section>

          <section className="sb-section">
            <div className="sb-title">Pinned</div>
            <div className="list">
              {filterChats(pinnedChats).map((c) => (
                <button key={c.id} className="list-item">
                  <div>
                    <div className="li-title">{c.title}</div>
                    <div className="li-sub">{c.paused ? "Paused" : "Active"} • {c.time}</div>
                  </div>
                </button>
              ))}
            </div>
          </section>

          <section className="sb-section">
            <div className="sb-title">Today</div>
            <div className="list">
              {filterChats(todayChats).map((c) => (
                <button key={c.id} className="list-item">
                  <div>
                    <div className="li-title">{c.title}</div>
                    <div className="li-sub">{c.time}</div>
                  </div>
                </button>
              ))}
            </div>
          </section>

          <section className="sb-section">
            <div className="sb-title">Discover</div>
            <div className="discover-card">
              <strong>Grow Your AI Ops</strong>
              <p>Templates, tools, memory packs and workflow blueprints.</p>
              <button className="btn">Explore</button>
            </div>
          </section>
        </div>
      )}

      <div className="sidebar-bottom">
        <div className="profile-pill">
          <div className="avatar">A</div>
          {!collapsed && <div><div className="li-title">Aryan</div><div className="li-sub">Founder • Pro</div></div>}
        </div>
      </div>
    </aside>
  );
}

function App(): JSX.Element {
  const [mode, setMode] = React.useState<AppMode>("normal");
  const [layout, setLayout] = React.useState(layoutStore.getModeConfig("normal"));

  React.useEffect(() => {
    applyThemeTokensToDocument(themeStore.getActiveProfile().tokens);
  }, []);

  const applyMode = (nextMode: AppMode) => {
    layoutStore.setMode(nextMode);
    setMode(nextMode);
    setLayout(layoutStore.getModeConfig(nextMode));
  };

  const resizePanel = (panel: "left" | "right", width: number) => {
    layoutStore.resizePanel(mode, panel, width);
    setLayout(layoutStore.getModeConfig(mode));
  };

  const toggleRight = () => {
    layoutStore.togglePanel(mode, "right");
    setLayout(layoutStore.getModeConfig(mode));
  };

  return (
    <AppShell
      body={
        <main className={`workspace ${mode === "mobile_focus" ? "workspace-mobile" : "workspace-desktop"}`}>
          {layout.panels.left.visible && (
            <ResizablePanel width={layout.panels.left.width} onWidthChange={(w) => resizePanel("left", w)} edge="right">
              <Sidebar mode={mode} onMode={applyMode} onToggleRight={toggleRight} rightOpen={layout.panels.right.visible} />
            </ResizablePanel>
          )}

          <PanelFrame title={mode === "coding" ? "Editor + Chat" : "Chat Window"}>
            {mode === "coding" ? (
              <div className="editor-mock">
                <div className="editor-tabs">
                  <span className="tab active">inbox/page.tsx</span>
                  <span className="tab">AgentChatWindow.tsx</span>
                  <span className="tab">Sidebar.tsx</span>
                </div>
                <pre className="editor-code">{`// VS-code style coding mode\n// chat assistant on right panel`}</pre>
              </div>
            ) : (
              <>
                <div className="chat-thread">
                  <div className="bubble ai">Inbox design ready. Sidebar is fully dynamic.</div>
                  <div className="bubble user">Good. Keep old + modern chat mix.</div>
                  <div className="bubble ai">Done. Workspace, project, history, discover and profile all in sidebar.</div>
                </div>
                <div className="composer">
                  <input className="input composer-input" placeholder="Message Cluaiz..." />
                  <button className="btn">Send</button>
                </div>
              </>
            )}
          </PanelFrame>

          {layout.panels.right.visible && mode !== "mobile_focus" && (
            <ResizablePanel width={layout.panels.right.width} onWidthChange={(w) => resizePanel("right", w)} edge="left">
              <PanelFrame title="Mission Control">
                <div className="insight-list">
                  <div className="insight-card"><h5>Context</h5><p>Selected project and active chat context.</p></div>
                  <div className="insight-card"><h5>Pause / Resume</h5><p>Control running flows and automation steps.</p></div>
                  <div className="insight-card"><h5>Actions</h5><p>Reply assist, summarize, route to workflow.</p></div>
                </div>
              </PanelFrame>
            </ResizablePanel>
          )}
        </main>
      }
    />
  );
}

createRoot(document.getElementById("root")!).render(<App />);
