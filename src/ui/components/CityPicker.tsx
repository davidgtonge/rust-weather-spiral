import type { AppEvent, CityOption } from "../../protocol/types";

type CityPickerProps = {
  cities: CityOption[];
  selectedId: string;
  onSelect: (event: AppEvent) => void;
};

export function CityPicker({ cities, selectedId, onSelect }: CityPickerProps) {
  return (
    <div className="city-picker" role="group" aria-label="City">
      {cities.map((city) => (
        <button
          key={city.id}
          type="button"
          className={city.id === selectedId ? "pill active" : "pill"}
          onClick={() => onSelect({ type: "citySelected", cityId: city.id })}
        >
          {city.label}
        </button>
      ))}
    </div>
  );
}
