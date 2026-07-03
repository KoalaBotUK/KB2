import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import HomeView from "./HomeView.vue";

// The landing page has several fixed-width (`w-96` = 384px) cards and a
// non-wrapping partner-logo row. On phone viewports (320-375px) these force
// the whole page to scroll horizontally instead of the cards shrinking to
// fit, and the four partner logos get squeezed into one line instead of
// wrapping.

describe("HomeView mobile responsiveness", () => {
  function mountView() {
    return mount(HomeView, {
      global: { stubs: { fa: true } },
    });
  }

  it("shrinks the hero call-to-action card on narrow viewports", () => {
    const wrapper = mountView();
    const card = wrapper.find(".card.glass");
    expect(card.exists()).toBe(true);
    expect(card.classes()).toContain("w-11/12");
  });

  it("shrinks the feature cards on narrow viewports", () => {
    const wrapper = mountView();
    const cards = wrapper.findAll(".card.bg-base-100");
    expect(cards.length).toBeGreaterThan(0);
    for (const card of cards) {
      expect(card.classes()).toContain("w-11/12");
    }
  });

  it("allows the partner logos row to wrap on narrow viewports", () => {
    const wrapper = mountView();
    const partnerLinks = wrapper.findAll('a[href="https://www.thenuel.com"]');
    expect(partnerLinks.length).toBe(1);
    const row = partnerLinks[0].element.parentElement;
    expect(row.classList.contains("flex-wrap")).toBe(true);
  });
});
