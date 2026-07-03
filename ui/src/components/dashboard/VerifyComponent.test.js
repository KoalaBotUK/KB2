import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import VerifyComponent from "./VerifyComponent.vue";
import { Guild, Vote, Verify } from "../../stores/guild.js";
import { GuildMeta } from "../../stores/meta.js";

// The verified-roles table (Role/Type/Pattern/Members/actions columns) is
// wider than common phone viewports and needs a horizontally scrollable
// wrapper so it doesn't force the whole page to scroll sideways on mobile.

describe("VerifyComponent mobile responsiveness", () => {
  let modalTarget;

  beforeEach(() => {
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  it("wraps the verified roles table in a horizontally scrollable container", () => {
    const wrapper = mount(VerifyComponent, {
      props: {
        guild: new Guild("guild-1", new Verify([], new Map()), new Vote([])),
        guildMeta: new GuildMeta("guild-1", "Test Guild", null, true, [], []),
      },
      global: {
        stubs: { fa: true, RoleSelect: true, RoleTag: true },
      },
      attachTo: document.body,
    });

    const table = wrapper.find("table.table");
    expect(table.exists()).toBe(true);
    expect(table.element.closest(".overflow-x-auto")).not.toBeNull();

    wrapper.unmount();
  });
});
