import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import axios from "axios";
import VerifyView from "./VerifyView.vue";
import { User } from "../../stores/user.js";
import { OauthToken } from "../../helpers/auth.js";

// The hero image only had `lg:w-80 lg:h-full` (explicitly flagged
// `<!-- fixme: responsive -->` in the source). Below the `lg` breakpoint it
// has no width constraint at all, so it renders at its huge natural
// intrinsic size and forces the whole page to scroll horizontally on phones
// and tablets.

vi.mock("axios");

function loggedInUser() {
  const token = new OauthToken();
  token.accessToken = "token-a";
  localStorage.setItem(
    "user",
    JSON.stringify({ userId: "user-1", links: [], linkGuilds: [], token }),
  );
  return new User("user-1", [], [], token);
}

describe("VerifyView mobile responsiveness", () => {
  beforeEach(() => {
    localStorage.clear();
    axios.get.mockImplementation((url) => {
      if (url.includes("/users/")) {
        return Promise.resolve({
          data: { user_id: "user-1", links: [], link_guilds: [] },
        });
      }
      return Promise.resolve({ data: [] });
    });
  });

  it("constrains the hero image width below the lg breakpoint", () => {
    loggedInUser();
    const wrapper = mount(VerifyView, {
      global: {
        stubs: {
          fa: true,
          DiscordAuthButton: true,
          LinkedAccountsTable: true,
          LinkAccountButton: true,
          LinkedGuildsSelect: true,
        },
      },
    });

    const img = wrapper.find("figure img");
    expect(img.exists()).toBe(true);
    expect(img.classes()).toContain("w-full");

    wrapper.unmount();
  });
});
