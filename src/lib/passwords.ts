import { randomInt } from "node:crypto";
import { wordlist } from "@scure/bip39/wordlists/english.js";

const LOWERCASE = "abcdefghijkmnopqrstuvwxyz";
const UPPERCASE = "ABCDEFGHJKLMNPQRSTUVWXYZ";
const NUMBERS = "23456789";
const SYMBOLS = "!@#$%^&*";

export type PasswordOptions = { numbers: boolean; symbols: boolean };

export function parsePasswordLength(value: string): number {
  if (!/^\d+$/.test(value.trim())) throw new Error("Enter a whole number from 5 to 64.");
  const length = Number(value);
  if (length < 5 || length > 64) throw new Error("Password length must be between 5 and 64.");
  return length;
}

function pick(characters: string): string {
  return characters[randomInt(characters.length)] ?? "";
}

function shuffle(values: string[]): string[] {
  for (let index = values.length - 1; index > 0; index -= 1) {
    const swapIndex = randomInt(index + 1);
    [values[index], values[swapIndex]] = [values[swapIndex] ?? "", values[index] ?? ""];
  }
  return values;
}

export function generatePassword(length: number, options: PasswordOptions): string {
  if (!Number.isInteger(length) || length < 5 || length > 64) {
    throw new Error("Password length must be between 5 and 64.");
  }
  const groups = [LOWERCASE, UPPERCASE];
  if (options.numbers) groups.push(NUMBERS);
  if (options.symbols) groups.push(SYMBOLS);
  const result = groups.map(pick);
  const fullCharacterSet = groups.join("");
  while (result.length < length) result.push(pick(fullCharacterSet));
  return shuffle(result).join("");
}

function tokenLength(modifier: string | undefined): number {
  if (!modifier) return 1;
  const length = Number(modifier);
  if (!Number.isInteger(length) || length < 1 || length > 64) {
    throw new Error("Token lengths must be whole numbers from 1 to 64.");
  }
  return length;
}

function randomCharacters(characters: string, length: number): string {
  return Array.from({ length }, () => pick(characters)).join("");
}

export function generateMemorablePassword(format: string): string {
  if (!format || format.length > 256) throw new Error("Provide a format up to 256 characters.");
  const usedWords = new Set<string>();
  const result = format.replace(/{(word|number|symbol|random)(?::(\w+))?}/g, (token, type, modifier) => {
    if (type === "word") {
      let word = "";
      do word = wordlist[randomInt(wordlist.length)] ?? "secure";
      while (usedWords.has(word) && usedWords.size < wordlist.length);
      usedWords.add(word);
      if (!modifier) return randomInt(2) === 0 ? word : word.toUpperCase();
      if (modifier === "uppercase") return word.toUpperCase();
      if (modifier === "lowercase") return word.toLowerCase();
      if (modifier === "capitalize") return word[0]?.toUpperCase() + word.slice(1);
      throw new Error(`Unsupported word modifier: ${modifier}`);
    }
    const length = tokenLength(modifier);
    if (type === "number") return randomCharacters(NUMBERS, length);
    if (type === "symbol") return randomCharacters(SYMBOLS, length);
    if (type === "random") return randomCharacters(LOWERCASE + UPPERCASE + NUMBERS + SYMBOLS, length);
    return token;
  });
  if (result.length < 5 || result.length > 128) {
    throw new Error("The generated password must contain 5 to 128 characters.");
  }
  return result;
}
