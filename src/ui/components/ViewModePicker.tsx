import type { AppEvent, ViewMode } from "../../protocol/types";

const VIEW_MODES: { id: ViewMode; label: string }[] = [
  { id: "metric", label: "Metric" },
  { id: "tapestry", label: "Tapestry" },
  { id: "ribbon", label: "Ribbon" },
  { id: "condition", label: "Condition" },
  { id: "glyphs", label: "Split Glyphs" },
  { id: "mandala", label: "Mandala" },
  { id: "fingerprint", label: "Fingerprint" },
  { id: "daylight", label: "Daylight" },
];

type ViewModePickerProps = {
  selected: ViewMode;
  description: string;
  onSelect: (event: AppEvent) => void;
};

export function ViewModePicker({ selected, description, onSelect }: ViewModePickerProps) {
  return (
    <div className="view-mode-picker">
      <p className="control-label">View</p>
      <div className="view-mode-tabs" role="tablist" aria-label="Visualization view">
        {VIEW_MODES.map((mode) => (
          <button
            key={mode.id}
            type="button"
            role="tab"
            aria-selected={mode.id === selected}
            className={mode.id === selected ? "tab active" : "tab"}
            onClick={() => onSelect({ type: "viewModeSelected", viewMode: mode.id })}
          >
            {mode.label}
          </button>
        ))}
      </div>
      <p className="view-mode-description">{description}</p>
    </div>
  );
}
