import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import axios from "axios";
import { User } from "../../stores/user.js";
import VerifyMicrosoftCallback from "./VerifyMicrosoftCallback.vue";

vi.mock("axios");

beforeEach(() => {
  vi.resetAllMocks();
  window.history.replaceState({}, "", "/verify/microsoft/callback?code=fixture-auth-code");
  localStorage.setItem("authFlow", JSON.stringify({
    redirectPath: "/verify/microsoft/callback",
    tokenUrl: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
    codeVerifier: "fixture-verifier",
  }));
  vi.spyOn(User, "loadCache").mockReturnValue({
    userId: "123",
    token: { accessToken: "fixture-discord-token" },
  });
  vi.spyOn(console, "warn").mockImplementation(() => {});
});

afterEach(() => {
  vi.restoreAllMocks();
  localStorage.clear();
  window.history.replaceState({}, "", "/");
});

it("ends the spinner and logs only the allowlisted token exchange failure", async () => {
  axios.post.mockRejectedValueOnce({
    config: { data: "fixture-auth-code" },
    response: { status: 400, data: { error: "invalid_grant", error_description: "private details" } },
  });

  const wrapper = mount(VerifyMicrosoftCallback);
  await flushPromises();

  expect(wrapper.find("[role=status]").exists()).toBe(false);
  expect(wrapper.get("[role=alert]").text()).toContain("Could not complete email authorization");
  expect(axios.post).toHaveBeenCalledTimes(1);
  expect(axios.post.mock.calls[0][2].timeout).toBe(15_000);
  expect(console.warn).toHaveBeenCalledWith("Verify failed", {
    provider: "microsoft",
    stage: "token_exchange",
    reason: "invalid_grant",
    status: 400,
    requestId: undefined,
  });
  wrapper.unmount();
});

it("ends the spinner on an empty API error and shows the Lambda request ID", async () => {
  const requestId = "b37656aa-a9f4-46e8-9022-928711feec0b";
  axios.post.mockResolvedValueOnce({ data: { access_token: "fixture-access-token", expires_in: 3600 } });
  axios.post.mockRejectedValueOnce({
    response: { status: 500, data: "", headers: { "x-kb-request-id": requestId } },
  });

  const wrapper = mount(VerifyMicrosoftCallback);
  await flushPromises();

  expect(wrapper.find("[role=status]").exists()).toBe(false);
  expect(wrapper.get("[role=alert]").text()).toContain("Could not link your email");
  expect(wrapper.text()).toContain(requestId);
  expect(axios.post.mock.calls[1][2].timeout).toBe(15_000);
  expect(console.warn).toHaveBeenCalledWith("Verify failed", {
    provider: "microsoft",
    stage: "link_request",
    reason: "http_error",
    status: 500,
    requestId,
  });
  wrapper.unmount();
});
