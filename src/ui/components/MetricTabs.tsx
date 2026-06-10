import type { AppEvent, Metric } from "../../protocol/types";

const METRICS: { id: Metric; label: string }[] = [
  { id: "cloud", label: "Cloud" },
  { id: "sunlight", label: "Sun" },
  { id: "rain", label: "Rain" },
  { id: "wind", label: "Wind" },
  { id: "temperature", label: "Temp" },
];

type MetricTabsProps = {
  selected: Metric;
  onSelect: (event: AppEvent) => void;
};

export function MetricTabs({ selected, onSelect }: MetricTabsProps) {
  return (
    <div className="metric-tabs" role="tablist" aria-label="Metric">
      {METRICS.map((metric) => (
        <button
          key={metric.id}
          type="button"
          role="tab"
          aria-selected={metric.id === selected}
          className={metric.id === selected ? "tab active" : "tab"}
          onClick={() => onSelect({ type: "metricSelected", metric: metric.id })}
        >
          {metric.label}
        </button>
      ))}
    </div>
  );
}
