import { Clipboard, Toast, getPreferenceValues, showHUD, showToast } from "@raycast/api";

import { generatePassword, parsePasswordLength } from "./lib/passwords";

type Preferences = { length?: string; useNumbers?: boolean; useSymbols?: boolean };

export default async function Command(): Promise<void> {
  const preferences = getPreferenceValues<Preferences>();
  try {
    const password = generatePassword(parsePasswordLength(preferences.length ?? "20"), {
      numbers: preferences.useNumbers ?? true,
      symbols: preferences.useSymbols ?? true,
    });
    await Clipboard.copy(password);
    await showHUD("Password copied");
  } catch (error) {
    await showToast(Toast.Style.Failure, "Could not generate password", String((error as Error).message));
  }
}
