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

import { generatePassword, parsePasswordLength } from "./lib/passwords";

type FormValues = { length: string; useNumbers: boolean; useSymbols: boolean };
type Preferences = {
  hideAfterCopy?: boolean;
  storePasswordLength?: boolean;
  rememberCharacterGroups?: boolean;
};

async function copyGeneratedPassword(values: FormValues): Promise<void> {
  let length: number;
  try {
    length = parsePasswordLength(values.length);
  } catch (error) {
    await showToast(Toast.Style.Failure, "Invalid password length", String((error as Error).message));
    return;
  }

  const password = generatePassword(length, {
    numbers: Boolean(values.useNumbers),
    symbols: Boolean(values.useSymbols),
  });
  await Clipboard.copy(password);

  const preferences = getPreferenceValues<Preferences>();
  if (preferences.hideAfterCopy) {
    await showHUD("Password copied", {
      clearRootSearch: false,
      popToRootType: PopToRootType.Suspended,
    });
  } else {
    await showToast(Toast.Style.Success, "Password copied", password);
  }
}

export default function Command() {
  const preferences = getPreferenceValues<Preferences>();

  return (
    <Form
      navigationTitle="Password Generator"
      actions={
        <ActionPanel>
          <Action.SubmitForm title="Generate and Copy Password" onSubmit={copyGeneratedPassword} />
        </ActionPanel>
      }
    >
      <Form.TextField
        id="length"
        title="Number of Characters"
        placeholder="20"
        defaultValue="20"
        storeValue={preferences.storePasswordLength ?? true}
      />
      <Form.Checkbox
        id="useNumbers"
        label="Use numbers"
        defaultValue
        storeValue={preferences.rememberCharacterGroups ?? true}
      />
      <Form.Checkbox
        id="useSymbols"
        label="Use symbols"
        defaultValue
        storeValue={preferences.rememberCharacterGroups ?? true}
      />
      <Form.Description text="Ambiguous characters such as 0, O, 1, and l are excluded." />
    </Form>
  );
}
