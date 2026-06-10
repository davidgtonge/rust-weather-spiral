import type { ViewModel } from "../../protocol/types";

type LegendProps = {
  vm: ViewModel;
};

function MetricLegend({ vm }: LegendProps) {
  return (
    <>
      <div className="legend-bar" aria-hidden="true" />
      <div className="legend-labels">
        <span>
          {vm.colorDomainMin}
          {vm.colorUnit}
        </span>
        <span>
          {vm.colorDomainMax}
          {vm.colorUnit}
        </span>
      </div>
    </>
  );
}

function TapestryLegend() {
  return (
    <ul className="legend-key">
      <li>
        <span className="swatch sky" /> Sky colour — sun vs cloud
      </li>
      <li>
        <span className="swatch rain" /> Blue ticks — rainfall
      </li>
      <li>
        <span className="swatch wind" /> Silver strokes — wind
      </li>
      <li>
        <span className="swatch storm" /> Dark emphasis — storms
      </li>
    </ul>
  );
}

function RibbonLegend() {
  return (
    <ul className="legend-key">
      <li>
        <span className="swatch sun" /> Track 1 — sunlight
      </li>
      <li>
        <span className="swatch cloud" /> Track 2 — cloud
      </li>
      <li>
        <span className="swatch rain" /> Track 3 — rain
      </li>
      <li>
        <span className="swatch wind" /> Track 4 — wind
      </li>
    </ul>
  );
}

function GlyphLegend() {
  return (
    <ul className="legend-key">
      <li>
        <span className="swatch sky" /> Upper block - sky blend from sun and cloud
      </li>
      <li>
        <span className="swatch rain" /> Blue drop - rainfall
      </li>
      <li>
        <span className="swatch temp" /> Lower block - temperature from cold to warm
      </li>
      <li>
        <span className="swatch wind" /> Silver strokes - wind strength
      </li>
    </ul>
  );
}

function ConditionLegend() {
  return (
    <ul className="legend-key legend-conditions">
      <li>
        <span className="swatch sunny" /> Sunny
      </li>
      <li>
        <span className="swatch cloudy" /> Cloudy
      </li>
      <li>
        <span className="swatch rainy" /> Rainy
      </li>
      <li>
        <span className="swatch windy" /> Windy
      </li>
      <li>
        <span className="swatch storm" /> Stormy
      </li>
      <li>
        <span className="swatch mixed" /> Mixed
      </li>
    </ul>
  );
}

export function Legend({ vm }: LegendProps) {
  return (
    <div className="legend">
      {vm.selectedViewMode === "metric" ? <MetricLegend vm={vm} /> : null}
      {vm.selectedViewMode === "tapestry" ||
      vm.selectedViewMode === "mandala" ||
      vm.selectedViewMode === "fingerprint" ? (
        <TapestryLegend />
      ) : null}
      {vm.selectedViewMode === "glyphs" ? <GlyphLegend /> : null}
      {vm.selectedViewMode === "daylight" ? (
        <ul className="legend-key">
          <li>
            <span className="swatch sky" /> Lit hour — sun &gt; 10 W/m²
          </li>
          <li>
            <span className="swatch rain" /> Angle = day of year
          </li>
          <li>
            <span className="swatch sun" /> Radius = dawn → dusk (seasonal band)
          </li>
          <li>Night is empty — compare Bristol vs Reykjavik</li>
        </ul>
      ) : null}
      {vm.selectedViewMode === "ribbon" ? <RibbonLegend /> : null}
      {vm.selectedViewMode === "condition" ? <ConditionLegend /> : null}
    </div>
  );
}
