import assert from "node:assert/strict";
import test from "node:test";
import { generateMemorablePassword, generatePassword, parsePasswordLength } from "../src/lib/passwords";

test("random passwords use the requested length and character groups", () => {
  const password = generatePassword(24, { numbers: true, symbols: true });
  assert.equal(password.length, 24);
  assert.match(password, /[a-z]/);
  assert.match(password, /[A-Z]/);
  assert.match(password, /[2-9]/);
  assert.match(password, /[!@#$%^&*]/);
});

test("password length validation rejects invalid values", () => {
  assert.equal(parsePasswordLength("20"), 20);
  assert.throws(() => parsePasswordLength("4"));
  assert.throws(() => parsePasswordLength("12.5"));
});

test("memorable formats replace every supported token", () => {
  const password = generateMemorablePassword("{word:capitalize}-{word}-{number:3}-{symbol:2}");
  assert.doesNotMatch(password, /[{}]/);
  assert.match(password, /-\d{3}-[!@#$%^&*]{2}$/);
});
