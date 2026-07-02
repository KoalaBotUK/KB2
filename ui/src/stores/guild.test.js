import { describe, expect, it } from "vitest";
import { VoteOption } from "./guild.js";

// Regression tests for issue #45: VoteOption.fromJson previously trusted
// `json['users']` to always be an array (it comes from a Rust HashSet
// serialized to JSON). If the field was missing or malformed, `users`
// would be `undefined`/non-array, and consumers calling `.length` on it
// (e.g. VoteComponent.vue) would throw. `fromJson` now normalizes any
// non-array `users` value to `[]`.
describe("VoteOption.fromJson", () => {
  it("passes a well-formed users array through unchanged", () => {
    const option = VoteOption.fromJson({
      emoji: "👍",
      label: "Yes",
      users: ["111", "222"],
    });

    expect(option.emoji).toBe("👍");
    expect(option.label).toBe("Yes");
    expect(option.users).toEqual(["111", "222"]);
    expect(Array.isArray(option.users)).toBe(true);
  });

  it("normalizes a missing users field to an empty array", () => {
    const option = VoteOption.fromJson({
      emoji: "👎",
      label: "No",
    });

    expect(option.users).toEqual([]);
    expect(option.users).not.toBeUndefined();
    expect(Array.isArray(option.users)).toBe(true);
  });

  it("normalizes a null users field to an empty array", () => {
    const option = VoteOption.fromJson({
      emoji: "🤷",
      label: "Maybe",
      users: null,
    });

    expect(option.users).toEqual([]);
    expect(Array.isArray(option.users)).toBe(true);
  });

  it("normalizes a non-array (string) users field to an empty array instead of passing it through raw", () => {
    const option = VoteOption.fromJson({
      emoji: "❓",
      label: "Unknown",
      users: "not-an-array",
    });

    expect(option.users).toEqual([]);
    expect(Array.isArray(option.users)).toBe(true);
    // Guards against the exact regression: raw non-array strings have a
    // .length too, so a naive guard could let this slip through unnormalized.
    expect(option.users).not.toBe("not-an-array");
  });
});
