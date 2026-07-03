import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import SystemComponent from "./SystemComponent.vue";

// The extension-info modal used a fixed `w-96` (384px) box. That's wider
// than common phone viewports (320-375px), so the modal content gets
// clipped/overflows on mobile. It needs to shrink on narrow screens
// (w-11/12) while still capping out at the original size on larger ones.

describe("SystemComponent mobile responsiveness", () => {
  let modalTarget;

  beforeEach(() => {
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  it("sizes the extension info modal to shrink on narrow viewports", async () => {
    const wrapper = mount(SystemComponent, {
      global: { stubs: { fa: true } },
      attachTo: document.body,
    });

    await wrapper.find("button.btn-sm").trigger("click");
    await wrapper.vm.$nextTick();

    const modalBox = document.querySelector(".modal-box");
    expect(modalBox).not.toBeNull();
    expect(modalBox.classList.contains("w-11/12")).toBe(true);

    wrapper.unmount();
  });
});
