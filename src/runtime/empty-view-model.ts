import type { ViewModel } from "../protocol/types";

export function emptyViewModel(): ViewModel {
  return {
    cityLabel: "",
    metricLabel: "",
    zoomLabel: "",
    viewModeLabel: "",
    viewModeDescription: "",
    cities: [],
    selectedCityId: "",
    selectedMetric: "cloud",
    selectedZoom: "year",
    selectedViewMode: "metric",
    showMetricTabs: true,
    frameWidth: 1024,
    frameHeight: 1024,
    colorDomainMin: 0,
    colorDomainMax: 100,
    colorUnit: "%",
    loading: true,
  };
}
