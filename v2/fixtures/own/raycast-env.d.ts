/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** After Copy - Return to the root after a password is copied. */
  "hideAfterCopy": boolean
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `generate-random-password` command */
  export type GenerateRandomPassword = ExtensionPreferences & {
  /** undefined - Remember the last password length between runs. */
  "storePasswordLength": boolean,
  /** undefined - Remember the selected character groups between runs. */
  "rememberCharacterGroups": boolean
}
  /** Preferences accessible in the `copy-random-password` command */
  export type CopyRandomPassword = ExtensionPreferences & {
  /** Number of Characters - Password length from 5 to 64 characters. */
  "length": string,
  /** Character Groups - Include numbers. */
  "useNumbers": boolean,
  /** undefined - Include special characters. */
  "useSymbols": boolean
}
  /** Preferences accessible in the `generate-memorable-password` command */
  export type GenerateMemorablePassword = ExtensionPreferences & {
  /** undefined - Remember the last passphrase format between runs. */
  "rememberCustomFormat": boolean
}
}

declare namespace Arguments {
  /** Arguments passed to the `generate-random-password` command */
  export type GenerateRandomPassword = {}
  /** Arguments passed to the `copy-random-password` command */
  export type CopyRandomPassword = {}
  /** Arguments passed to the `generate-memorable-password` command */
  export type GenerateMemorablePassword = {}
}

