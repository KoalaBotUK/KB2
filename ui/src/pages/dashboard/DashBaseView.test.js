import { describe, it, expect } from "vitest";
import rawSource from "./DashBaseView.vue?raw";
import { parseComponentTemplate } from "../../testUtils/templateDom.js";

// The dashboard header navbar breaks on phone viewports:
// - The current-guild button had no width cap or truncation, so a normal
//   length server name (e.g. "Koala Bots") wraps onto two lines using the
//   oversized `card-title` font, roughly doubling the header's height.
// - navbar-end had a flat `px-10` (80px) of horizontal padding, which eats
//   a large chunk of a ~375-414px viewport and crowds everything else.
// - The center logo doesn't reserve its own space in the navbar (DaisyUI's
//   navbar-start/navbar-end both stretch to fill the remaining width), so
//   on narrow screens it visually collides with the account button next to
//   it. It's decorative only (the koala branding is already in the browser
//   tab and the page footer), so it's hidden below the sm breakpoint rather
//   than fought over.
//
// DashBaseView has heavy onMounted data-loading side effects, so rather
// than fully mounting it we parse its raw <template> to assert on the
// structure Vue will render.

describe("DashBaseView mobile responsiveness", () => {
  it("truncates the current-guild name instead of letting it wrap", () => {
    const dom = parseComponentTemplate(rawSource);
    const nameLabel = Array.from(dom.querySelectorAll("*")).find(
      (el) =>
        el.children.length === 0 &&
        el.textContent.includes("guildMetaMap.get(currentGuildId).name"),
    );
    expect(nameLabel).toBeTruthy();
    expect(nameLabel.classList.contains("truncate")).toBe(true);

    const guildButton = nameLabel.closest('[role="button"]');
    expect(guildButton).toBeTruthy();
    expect(guildButton.className).toMatch(/max-w-/);
  });

  it("shrinks the navbar-end padding on narrow viewports", () => {
    const dom = parseComponentTemplate(rawSource);
    const navbarEnd = dom.querySelector(".navbar-end");
    expect(navbarEnd).toBeTruthy();
    expect(navbarEnd.classList.contains("px-2")).toBe(true);
    expect(navbarEnd.classList.contains("sm:px-10")).toBe(true);
  });

  it("hides the decorative center logo on narrow viewports so it can't collide with the account button", () => {
    const dom = parseComponentTemplate(rawSource);
    const navbarCenter = dom.querySelector(".navbar-center");
    expect(navbarCenter).toBeTruthy();
    expect(navbarCenter.classList.contains("hidden")).toBe(true);
    expect(navbarCenter.classList.contains("sm:flex")).toBe(true);
  });
});
