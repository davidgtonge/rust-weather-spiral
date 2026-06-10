import type { AppEvent, Zoom } from "../../protocol/types";

const ZOOMS: { id: Zoom; label: string }[] = [
  { id: "year", label: "Year" },
  { id: "month", label: "Month" },
  { id: "week", label: "Week" },
  { id: "day", label: "Day" },
];

type ZoomControlProps = {
  selected: Zoom;
  onSelect: (event: AppEvent) => void;
};

export function ZoomControl({ selected, onSelect }: ZoomControlProps) {
  return (
    <div className="zoom-control" role="group" aria-label="Zoom">
      {ZOOMS.map((zoom) => (
        <button
          key={zoom.id}
          type="button"
          className={zoom.id === selected ? "segment active" : "segment"}
          onClick={() => onSelect({ type: "zoomSelected", zoom: zoom.id })}
        >
          {zoom.label}
        </button>
      ))}
    </div>
  );
}
