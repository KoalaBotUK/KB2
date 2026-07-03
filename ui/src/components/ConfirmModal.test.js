import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ConfirmModal from "./ConfirmModal.vue";

// ConfirmModal used a fixed `w-96` (384px) modal box, wider than common
// phone viewports (320-375px), clipping the confirm/cancel buttons on
// mobile. It needs to shrink on narrow screens (w-11/12) while capping out
// at the original size on larger ones.

describe("ConfirmModal mobile responsiveness", () => {
  let modalTarget;

  beforeEach(() => {
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    modalTarget.remove();
  });

  it("sizes the modal box to shrink on narrow viewports", () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        title: "Delete?",
        confirmClass: "btn-error",
        confirmText: "confirm",
        activeEvent: { target: { id: "1" } },
      },
      attachTo: document.body,
    });

    const modalBox = document.querySelector(".modal-box");
    expect(modalBox).not.toBeNull();
    expect(modalBox.classList.contains("w-11/12")).toBe(true);

    wrapper.unmount();
  });
});
