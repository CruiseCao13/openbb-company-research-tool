import { type ReactNode, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type StudioPing = {
  status: "ok";
  message: string;
};

type BadgeVariant = "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "EXTERNAL_AI" | "LOCAL_MOCK" | "UNKNOWN";

type ResearchCardConfig = {
  title: string;
  badge: BadgeVariant;
  body: string;
};

const placeholderCards: ResearchCardConfig[] = [
  {
    title: "Report Status",
    badge: "UNKNOWN",
    body: "Placeholder only. No report status has been loaded from a v5 run folder."
  },
  {
    title: "AI Source",
    badge: "EXTERNAL_AI",
    body: "Placeholder only. Future DTOs will show external, local, cache, and skipped provenance."
  },
  {
    title: "Company Identity",
    badge: "UNKNOWN",
    body: "Placeholder only. Company profile and research frame are intentionally not loaded in Phase 2."
  },
  {
    title: "Money Flow",
    badge: "DATA_GAP",
    body: "Placeholder only. Sources, uses, cash-flow gaps, and financing pressure will appear after run loading exists."
  },
  {
    title: "Data Gaps",
    badge: "WARNING",
    body: "Placeholder only. Missing provider fields and unsupported claims will be rendered from typed DTOs later."
  }
];

function StatusBadge({ variant }: { variant: BadgeVariant }): JSX.Element {
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function EmptyState({ title, detail }: { title: string; detail: string }): JSX.Element {
  return (
    <div className="empty-state">
      <strong>{title}</strong>
      <span>{detail}</span>
    </div>
  );
}

function Sidebar(): JSX.Element {
  return (
    <aside className="sidebar" aria-label="Run navigation">
      <div className="brand-block">
        <p className="eyebrow">v6 studio shell</p>
        <h1>v6 Tauri Research Studio</h1>
        <p>Desktop research workspace for v5 run folders</p>
      </div>

      <section className="panel">
        <div className="panel-header">
          <span>Runs</span>
          <StatusBadge variant="UNKNOWN" />
        </div>
        <EmptyState title="No run selected" detail="Waiting for run discovery" />
      </section>
    </aside>
  );
}

function TopStatusStrip({ ipcMessage }: { ipcMessage: string }): JSX.Element {
  return (
    <header className="top-status-strip" aria-label="Studio status">
      <div className="status-cluster">
        <span className="status-dot" aria-hidden="true" />
        <span>Studio shell ready</span>
      </div>
      <span>No external API used</span>
      <span>No run loaded</span>
      <span className="ipc-readout">{ipcMessage}</span>
    </header>
  );
}

function ResearchCard({ card }: { card: ResearchCardConfig }): JSX.Element {
  return (
    <article className="detail-card">
      <div className="card-header">
        <span className="card-label">{card.title}</span>
        <StatusBadge variant={card.badge} />
      </div>
      <p>{card.body}</p>
    </article>
  );
}

function BottomProvenanceBar(): JSX.Element {
  return (
    <section className="bottom-bar" aria-label="Provenance and data gaps">
      <div className="bottom-bar__header">
        <h3>Provenance &amp; Data Gaps</h3>
        <StatusBadge variant="DATA_GAP" />
      </div>
      <div className="bottom-bar__grid">
        <div className="provenance-cell">
          <span>AI provenance</span>
          <p>AI provenance will appear here</p>
        </div>
        <div className="provenance-cell">
          <span>Data gaps</span>
          <p>Data gaps will appear here</p>
        </div>
      </div>
    </section>
  );
}

function AppShell({ children }: { children: ReactNode }): JSX.Element {
  return <main className="studio-shell">{children}</main>;
}

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
    <AppShell>
      <Sidebar />

      <section className="workspace" aria-label="Research run detail">
        <TopStatusStrip ipcMessage={ipcMessage} />

        <header className="workspace-header">
          <div>
            <p className="eyebrow">Static placeholder layout</p>
            <h2>Research Run Detail</h2>
            <p>Select a run to inspect locked data, AI provenance, validator logs, and report artifacts.</p>
          </div>
        </header>

        <section className="card-grid" aria-label="Placeholder detail cards">
          {placeholderCards.map((card) => (
            <ResearchCard card={card} key={card.title} />
          ))}
        </section>

        <BottomProvenanceBar />
      </section>
    </AppShell>
  );
}
