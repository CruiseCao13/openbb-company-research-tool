import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type StudioPing = {
  status: "ok";
  message: string;
};

const placeholderCards = [
  "Report Status",
  "AI Source",
  "Company Identity",
  "Money Flow",
  "Data Gaps"
] as const;

export function App(): JSX.Element {
  const [ipcMessage, setIpcMessage] = useState<string>("IPC not checked");

  useEffect(() => {
    let mounted = true;

    invoke<StudioPing>("ping_studio")
      .then((response) => {
        if (mounted) {
          setIpcMessage(`${response.status}: ${response.message}`);
        }
      })
      .catch(() => {
        if (mounted) {
          setIpcMessage("IPC unavailable in browser preview");
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  return (
    <main className="studio-shell">
      <aside className="sidebar" aria-label="Run navigation">
        <div className="brand-block">
          <p className="eyebrow">Phase 1 shell</p>
          <h1>v6 Tauri Research Studio</h1>
          <p>Desktop research workspace for v5 run folders</p>
        </div>

        <section className="panel">
          <div className="panel-header">
            <span>Runs</span>
            <span className="status-pill">Placeholder</span>
          </div>
          <div className="empty-state">No run selected</div>
        </section>
      </aside>

      <section className="workspace" aria-label="Research run detail">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">Read-only studio shell</p>
            <h2>Research Run Detail</h2>
          </div>
          <div className="ipc-status">
            <span>Backend</span>
            <strong>{ipcMessage}</strong>
          </div>
        </header>

        <section className="card-grid" aria-label="Placeholder detail cards">
          {placeholderCards.map((title) => (
            <article className="detail-card" key={title}>
              <div className="card-label">{title}</div>
              <p>Placeholder only. Waiting for a typed DTO from a future read-only Tauri command.</p>
            </article>
          ))}
        </section>

        <section className="bottom-bar" aria-label="Provenance and data gaps">
          <div>
            <h3>Provenance &amp; Data Gaps</h3>
            <p>Waiting for selected run</p>
          </div>
          <div className="status-line">
            <span>Studio shell ready</span>
            <span>No external API used</span>
          </div>
        </section>
      </section>
    </main>
  );
}
