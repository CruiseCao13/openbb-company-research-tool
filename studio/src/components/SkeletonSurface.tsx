type SkeletonSurfaceProps = {
  detail?: string;
  title: string;
  variant?: "flow" | "lines" | "gauges";
};

export function SkeletonLine({ width = "100%" }: { width?: string }): JSX.Element {
  return <span className="skeleton-line" style={{ width }} />;
}

export function SkeletonGauge(): JSX.Element {
  return (
    <div className="skeleton-gauge" aria-hidden="true">
      <span />
    </div>
  );
}

export function SkeletonFlow(): JSX.Element {
  return (
    <div className="skeleton-flow" aria-hidden="true">
      <span className="skeleton-flow__river skeleton-flow__river--main" />
      <span className="skeleton-flow__river skeleton-flow__river--branch" />
      <span className="skeleton-flow__node skeleton-flow__node--left" />
      <span className="skeleton-flow__node skeleton-flow__node--mid" />
      <span className="skeleton-flow__node skeleton-flow__node--right" />
    </div>
  );
}

export function SkeletonSurface({ detail, title, variant = "lines" }: SkeletonSurfaceProps): JSX.Element {
  return (
    <section className={`skeleton-crystal skeleton-crystal--${variant}`} aria-label={title} aria-busy="true">
      {variant === "flow" ? <SkeletonFlow /> : null}
      {variant === "gauges" ? (
        <div className="skeleton-gauge-grid">
          <SkeletonGauge />
          <SkeletonGauge />
          <SkeletonGauge />
        </div>
      ) : null}
      <div className="skeleton-copy">
        <strong>{title}</strong>
        {detail ? <span>{detail}</span> : null}
        <SkeletonLine width="72%" />
        <SkeletonLine width="48%" />
      </div>
    </section>
  );
}
