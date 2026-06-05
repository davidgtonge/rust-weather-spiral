import { createBuiltinEffectRegistry } from "@dtonge/engine-shell";
import type { AppEvent, EffectCommand } from "./protocol/types";

export const weatherEffects = createBuiltinEffectRegistry<EffectCommand, AppEvent>({
  match: (effect) => {
    if (effect.type === "timerStart" || effect.type === "timerStop") {
      return effect;
    }
    return null;
  },
  onTimerTick: () => ({ type: "tick" }),
  onRandomInt: () => ({ type: "tick" }),
});
