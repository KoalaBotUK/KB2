import { describe, it, expect, vi, beforeEach } from "vitest";
import axios from "axios";
import { User } from "../stores/user.js";
import { linkEmail } from "./user.js";

// Regression test for issue #48: "Store modules capture stale user at import
// time via module-level User.loadCache() snapshot".
//
// Before the fix, `let user = User.loadCache()` sat at module scope, so the
// value returned by `User.loadCache()` at the moment the module was first
// imported was captured once and reused forever - later changes to the
// cached user (re-login, token refresh, logout/login as someone else) were
// silently ignored by every call into the module. The fix moved the
// `User.loadCache()` call inside each function body so it is read fresh on
// every invocation.
//
// This test drives `linkEmail` twice within a single test, changing what
// `User.loadCache()` returns between the two calls, and asserts that the
// *second* call picks up the *new* value rather than the one returned by
// the first call.

vi.mock("axios");

describe("linkEmail", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    axios.post.mockResolvedValue({ data: {} });
  });

  it("reads User.loadCache() fresh on every call instead of a stale snapshot", async () => {
    const userA = { userId: "user-a", token: { accessToken: "token-a" } };
    const userB = { userId: "user-b", token: { accessToken: "token-b" } };

    const loadCacheSpy = vi.spyOn(User, "loadCache");

    loadCacheSpy.mockReturnValueOnce(userA);
    await linkEmail("google", "tok1");

    expect(axios.post).toHaveBeenNthCalledWith(
      1,
      expect.stringContaining(`/users/${userA.userId}/links`),
      expect.objectContaining({ origin: "google", token: "tok1" }),
      expect.objectContaining({
        headers: { Authorization: `Discord ${userA.token.accessToken}` },
      }),
    );

    // Simulate the cached user changing between calls (e.g. a fresh
    // login). If `User.loadCache()` were only read once at import time,
    // this second call would still behave as if `userA` were current.
    loadCacheSpy.mockReturnValueOnce(userB);
    await linkEmail("google", "tok2");

    expect(axios.post).toHaveBeenNthCalledWith(
      2,
      expect.stringContaining(`/users/${userB.userId}/links`),
      expect.objectContaining({ origin: "google", token: "tok2" }),
      expect.objectContaining({
        headers: { Authorization: `Discord ${userB.token.accessToken}` },
      }),
    );

    expect(loadCacheSpy).toHaveBeenCalledTimes(2);
  });
});
