import { describe, it, expect, vi, beforeEach } from "vitest";
import axios from "axios";
import { User } from "./user.js";
import { Guild, VoteOption } from "./guild.js";

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

// Regression test for issue #48: "Store modules capture stale user at import
// time via module-level User.loadCache() snapshot".
//
// `Guild.loadGuild` previously relied on a module-level `let user =
// User.loadCache()` captured once when the store module was first imported,
// so subsequent changes to the cached user (e.g. a re-login) were never
// reflected in outgoing requests. The fix reads `User.loadCache()` inside
// the function body on every call.
//
// This test calls `Guild.loadGuild` twice, changing the mocked cached user
// in between, and asserts each request uses the *current* user rather than
// whichever user happened to be cached when the module was first loaded.

vi.mock("axios");

const emptyGuildResponse = {
  data: {
    guild_id: "g1",
    verify: { roles: [], user_links: [] },
    vote: { votes: [] },
  },
};

describe("Guild.loadGuild", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    axios.post.mockResolvedValue(emptyGuildResponse);
  });

  it("uses the current cached user on each call instead of a stale snapshot", async () => {
    const userA = { token: { accessToken: "token-a" } };
    const userB = { token: { accessToken: "token-b" } };

    const loadCacheSpy = vi.spyOn(User, "loadCache");

    loadCacheSpy.mockReturnValueOnce(userA);
    await Guild.loadGuild("g1");

    expect(axios.post).toHaveBeenNthCalledWith(
      1,
      expect.stringContaining("/guilds/g1"),
      {},
      expect.objectContaining({
        headers: { Authorization: `Discord ${userA.token.accessToken}` },
      }),
    );

    // Simulate the cached user changing between calls.
    loadCacheSpy.mockReturnValueOnce(userB);
    await Guild.loadGuild("g1");

    expect(axios.post).toHaveBeenNthCalledWith(
      2,
      expect.stringContaining("/guilds/g1"),
      {},
      expect.objectContaining({
        headers: { Authorization: `Discord ${userB.token.accessToken}` },
      }),
    );

    expect(loadCacheSpy).toHaveBeenCalledTimes(2);
  });
});
