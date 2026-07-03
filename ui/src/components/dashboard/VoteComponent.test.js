import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import VoteComponent from "./VoteComponent.vue";
import { User } from "../../stores/user.js";
import { Guild, Vote, Verify } from "../../stores/guild.js";
import { GuildMeta } from "../../stores/meta.js";

// Regression test for issue #46: the Blacklist/Whitelist radios in the
// "Role List" section of the create-vote modal previously were not a real
// v-model pair, so selecting "Whitelist" never flipped `modelBlacklist`
// back to false. This test drives the actual radio inputs and asserts the
// shared model updates in both directions.

function makeProps(initialBlacklist) {
  return {
    user: new User("user-1", [], [], null),
    guild: new Guild("guild-1", new Verify([], new Map()), new Vote([]), []),
    guildMeta: new GuildMeta("guild-1", "Test Guild", null, true, [], []),
    modelBlacklist: initialBlacklist,
  };
}

describe("VoteComponent Blacklist/Whitelist radio pair", () => {
  let modalTarget;

  beforeEach(() => {
    // The radios live inside <Teleport to="#modal">, so a real target
    // element must exist in the document for them to be reachable.
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  function mountComponent(initialBlacklist) {
    return mount(VoteComponent, {
      props: makeProps(initialBlacklist),
      global: {
        stubs: {
          fa: true,
          ChannelSelect: true,
          RoleSelect: true,
        },
      },
      attachTo: document.body,
    });
  }

  function getRadios() {
    return Array.from(
      document.querySelectorAll('input[name="roleListType"]'),
    );
  }

  it("selecting Whitelist sets modelBlacklist to false", async () => {
    const wrapper = mountComponent(true);
    // Open the create-vote modal so the radios are rendered/visible.
    await wrapper.find("button.btn-primary.btn-sm").trigger("click");
    await wrapper.vm.$nextTick();

    const radios = getRadios();
    expect(radios).toHaveLength(2);
    const blacklistRadio = radios.find(
      (r) => r.getAttribute("aria-label") === "Blacklist",
    );
    const whitelistRadio = radios.find(
      (r) => r.getAttribute("aria-label") === "Whitelist",
    );
    expect(blacklistRadio).toBeTruthy();
    expect(whitelistRadio).toBeTruthy();

    whitelistRadio.checked = true;
    whitelistRadio.dispatchEvent(new Event("change"));
    await wrapper.vm.$nextTick();

    const emitted = wrapper.emitted("update:modelBlacklist");
    expect(emitted).toBeTruthy();
    expect(emitted.at(-1)[0]).toBe(false);

    wrapper.unmount();
  });

  it("selecting Blacklist sets modelBlacklist to true", async () => {
    const wrapper = mountComponent(false);
    await wrapper.find("button.btn-primary.btn-sm").trigger("click");
    await wrapper.vm.$nextTick();

    const radios = getRadios();
    const blacklistRadio = radios.find(
      (r) => r.getAttribute("aria-label") === "Blacklist",
    );

    blacklistRadio.checked = true;
    blacklistRadio.dispatchEvent(new Event("change"));
    await wrapper.vm.$nextTick();

    const emitted = wrapper.emitted("update:modelBlacklist");
    expect(emitted).toBeTruthy();
    expect(emitted.at(-1)[0]).toBe(true);

    wrapper.unmount();
  });
});
