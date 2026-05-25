import { useTranslation } from "react-i18next";

export type LandingMarket = "auto" | "us" | "cn";
export type LandingAnalysisMode = "load_existing" | "run_local" | "external_disabled";

export const exampleTickers = ["AAPL", "GOOGL", "RKLB", "600519.SH", "JPM"];

export function LandingExperience({
  landingAnalysisMode,
  landingMarket,
  landingSearch,
  matchingRunsCount,
  onEnter,
  onLandingSearch,
  onOpenLatest,
  onOpenMatrix,
  onSetAnalysisMode,
  onSetMarket
}: {
  landingAnalysisMode: LandingAnalysisMode;
  landingMarket: LandingMarket;
  landingSearch: string;
  matchingRunsCount: number;
  onEnter: () => void;
  onLandingSearch: (value: string) => void;
  onOpenLatest: () => void;
  onOpenMatrix: () => void;
  onSetAnalysisMode: (value: LandingAnalysisMode) => void;
  onSetMarket: (value: LandingMarket) => void;
}): JSX.Element {
  const { t } = useTranslation();

  return (
    <section className="landing-hero" aria-label="Studio landing">
      <div className="hero-orbit" aria-hidden="true"><span /><span /><span /></div>
      <div className="landing-hero__canvas" aria-label="Example Preview vascular money flow">
        <ExampleVascularPreview />
      </div>
      <div className="landing-hero__content">
        <p className="eyebrow">{t("examplePreview")}</p>
        <h2>{t("heroTitle")}</h2>
        <p>{t("heroSubtitle")}</p>
        <div className="example-ticker-strip" aria-label="Example companies">
          {exampleTickers.map((ticker) => (
            <button key={ticker} onClick={() => onLandingSearch(ticker)} type="button">
              {ticker}
            </button>
          ))}
        </div>
      </div>
      <form
        className="landing-search-console"
        onSubmit={(event) => {
          event.preventDefault();
          onEnter();
        }}
      >
        <label>
          <span>{t("searchTicker")}</span>
          <input
            autoComplete="off"
            onChange={(event) => onLandingSearch(event.target.value)}
            placeholder={t("searchTickerPlaceholder")}
            value={landingSearch}
          />
        </label>
        <div className="landing-search-console__controls">
          <select aria-label={t("market")} onChange={(event) => onSetMarket(event.target.value as LandingMarket)} value={landingMarket}>
            <option value="auto">{t("auto")}</option>
            <option value="us">US</option>
            <option value="cn">CN_A</option>
          </select>
          <select
            aria-label={t("analysisMode")}
            onChange={(event) => onSetAnalysisMode(event.target.value as LandingAnalysisMode)}
            value={landingAnalysisMode}
          >
            <option value="load_existing">{t("loadExistingRun")}</option>
            <option value="run_local" disabled>{t("runLocalComingNext")}</option>
            <option value="external_disabled" disabled>{t("externalDisabled")}</option>
          </select>
        </div>
        <div className="landing-search-console__actions">
          <button className="hero-cta hero-cta--primary" type="submit">{t("analyze")}</button>
          <button className="hero-cta" onClick={onOpenLatest} type="button">{t("loadLatest")}</button>
          <button className="hero-cta" onClick={onOpenMatrix} type="button">{t("viewMatrix")}</button>
        </div>
        {landingSearch.trim().length > 0 && matchingRunsCount === 0 ? (
          <div className="landing-no-result">
            <strong>{t("noExistingRunFound")}</strong>
            <span>{t("runLocalComingNext")}</span>
          </div>
        ) : null}
        <p>{t("localOnlyNotice")}</p>
      </form>
    </section>
  );
}

function ExampleVascularPreview(): JSX.Element {
  const { t } = useTranslation();
  return (
    <div className="vascular-demo" aria-label={t("examplePreview")}>
      <svg viewBox="0 0 900 520" role="img" aria-label="Example qualitative vascular money flow preview">
        <defs>
          <filter id="vascular-glow" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="7" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          <linearGradient id="vascular-green" x1="0%" x2="100%" y1="0%" y2="0%">
            <stop offset="0%" stopColor="rgba(52, 211, 153, 0.1)" />
            <stop offset="48%" stopColor="rgba(52, 211, 153, 0.88)" />
            <stop offset="100%" stopColor="rgba(143, 211, 255, 0.78)" />
          </linearGradient>
          <linearGradient id="vascular-blue" x1="0%" x2="100%" y1="0%" y2="0%">
            <stop offset="0%" stopColor="rgba(143, 211, 255, 0.14)" />
            <stop offset="55%" stopColor="rgba(143, 211, 255, 0.82)" />
            <stop offset="100%" stopColor="rgba(96, 165, 250, 0.36)" />
          </linearGradient>
          <linearGradient id="vascular-risk" x1="0%" x2="100%" y1="0%" y2="0%">
            <stop offset="0%" stopColor="rgba(251, 146, 60, 0.18)" />
            <stop offset="62%" stopColor="rgba(251, 146, 60, 0.72)" />
            <stop offset="100%" stopColor="rgba(244, 63, 94, 0.5)" />
          </linearGradient>
        </defs>
        <path className="vascular-path vascular-path--main" d="M90 250 C190 120 300 170 400 235 C500 300 620 270 790 120" />
        <path className="vascular-path vascular-path--blue" d="M400 235 C470 160 570 120 710 210" />
        <path className="vascular-path vascular-path--risk" d="M405 250 C485 330 610 360 780 405" />
        <path className="vascular-path vascular-path--thin" d="M190 260 C275 330 350 348 485 385" />
        {[
          [t("flowNodeRevenue"), 90, 250],
          [t("flowNodeCashEngine"), 400, 235],
          [t("flowNodeReinvestment"), 710, 210],
          [t("flowNodeFreeCash"), 790, 120],
          [t("flowNodeRiskGaps"), 780, 405]
        ].map(([label, x, y]) => (
          <g className="vascular-node" key={label as string}>
            <circle cx={x as number} cy={y as number} r="16" />
            <text x={(x as number) + 26} y={(y as number) + 4}>{label}</text>
          </g>
        ))}
      </svg>
      <div className="vascular-demo__badge">
        <span>{t("examplePreview")}</span>
        <strong>{t("qualitativeFlow")}</strong>
      </div>
    </div>
  );
}
