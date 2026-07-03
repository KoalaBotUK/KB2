import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import DiscordAuthButton from "./DiscordAuthButton.vue";
import { User } from "../../stores/user.js";
import { OauthToken } from "../../helpers/auth.js";
import { UserMeta } from "../../stores/meta.js";

// The logged-in account modal used a fixed `w-96` (384px) box, wider than
// common phone viewports (320-375px), clipping the username/logout button
// on mobile. It needs to shrink on narrow screens (w-11/12) while capping
// out at the original size on larger ones.

function loggedInUser() {
  const token = new OauthToken();
  token.accessToken = "token-a";
  return new User("user-1", [], [], token);
}

describe("DiscordAuthButton mobile responsiveness", () => {
  let modalTarget;

  beforeEach(() => {
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  it("sizes the account modal to shrink on narrow viewports", async () => {
    const wrapper = mount(DiscordAuthButton, {
      props: {
        user: loggedInUser(),
        userMeta: new UserMeta("user-1", "Test User", null),
      },
      attachTo: document.body,
    });

    await wrapper.find("button.btn").trigger("click");
    await wrapper.vm.$nextTick();

    const modalBox = document.querySelector(".modal-box");
    expect(modalBox).not.toBeNull();
    expect(modalBox.classList.contains("w-11/12")).toBe(true);

    wrapper.unmount();
  });
});
