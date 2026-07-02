import { describe, it, expect, vi, beforeEach } from "vitest";
import axios from "axios";
import { User } from "./user.js";
import { Guild } from "./guild.js";

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
