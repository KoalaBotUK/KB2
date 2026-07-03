import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import VoteComponent from "./VoteComponent.vue";
import { User } from "../../stores/user.js";
import { Guild, Vote, Verify } from "../../stores/guild.js";
import { GuildMeta } from "../../stores/meta.js";

// Mobile responsiveness regression tests.
//
// On narrow viewports the votes table (5 columns: Title, Description, State,
// Voters, actions) is wider than the screen. Without a horizontally
// scrollable wrapper, the table forces the whole page to scroll sideways
// instead of just the table. Similarly the create/results modals used a
// fixed `w-96` (384px) box which is wider than common phone viewports
// (320-375px), clipping the modal content.

function makeProps() {
  return {
    user: new User("user-1", [], [], null),
    guild: new Guild("guild-1", new Verify([], new Map()), new Vote([])),
    guildMeta: new GuildMeta("guild-1", "Test Guild", null, true, [], []),
  };
}

function mountComponent() {
  return mount(VoteComponent, {
    props: makeProps(),
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

describe("VoteComponent mobile responsiveness", () => {
  let modalTarget;

  beforeEach(() => {
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  it("wraps the votes table in a horizontally scrollable container", () => {
    const wrapper = mountComponent();
    const table = wrapper.find("table.table");
    expect(table.exists()).toBe(true);
    expect(table.element.closest(".overflow-x-auto")).not.toBeNull();
    wrapper.unmount();
  });

  it("sizes the create-vote modal to shrink on narrow viewports", async () => {
    const wrapper = mountComponent();
    await wrapper.find("button.btn-primary.btn-sm").trigger("click");
    await wrapper.vm.$nextTick();

    const modalBoxes = document.querySelectorAll(".modal-box");
    expect(modalBoxes.length).toBeGreaterThan(0);
    for (const box of modalBoxes) {
      expect(box.classList.contains("w-11/12")).toBe(true);
    }

    wrapper.unmount();
  });
});
