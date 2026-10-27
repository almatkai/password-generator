import {
  Action,
  ActionPanel,
  Clipboard,
  Form,
  PopToRootType,
  Toast,
  getPreferenceValues,
  showHUD,
  showToast,
} from "@raycast/api";

import { generateMemorablePassword } from "./lib/passwords";

type FormValues = { format: string };
type Preferences = { hideAfterCopy?: boolean; rememberCustomFormat?: boolean };

async function copyPassphrase(values: FormValues): Promise<void> {
  const format = values.format.trim() || "{word:capitalize}-{word}-{word}-{number:3}";
  try {
    const password = generateMemorablePassword(format);
    await Clipboard.copy(password);
    const preferences = getPreferenceValues<Preferences>();
    if (preferences.hideAfterCopy) {
      await showHUD("Memorable password copied", {
        clearRootSearch: false,
        popToRootType: PopToRootType.Suspended,
      });
    } else {
      await showToast(Toast.Style.Success, "Memorable password copied", password);
    }
  } catch (error) {
    await showToast(Toast.Style.Failure, "Invalid format", String((error as Error).message));
  }
}

export default function Command() {
  const preferences = getPreferenceValues<Preferences>();
  return (
    <Form
      navigationTitle="Memorable Password"
      actions={
        <ActionPanel>
          <Action.SubmitForm title="Generate and Copy Password" onSubmit={copyPassphrase} />
        </ActionPanel>
      }
    >
      <Form.TextField
        id="format"
        title="Format"
        defaultValue="{word:capitalize}-{word}-{word}-{number:3}"
        placeholder="{word:capitalize}-{word}-{number:3}"
        storeValue={preferences.rememberCustomFormat ?? true}
      />
      <Form.Description
        text={
          "Tokens: {word}, {word:uppercase}, {word:lowercase}, {word:capitalize}, " +
          "{number:n}, {symbol:n}, and {random:n}."
        }
      />
    </Form>
  );
}
