import { describe, it, expect } from "vitest";
import rawSource from "./AnnounceComponent.vue?raw";
import { parseComponentTemplate } from "../../testUtils/templateDom.js";

// The announcements table (Title/Members/Sent/actions columns) is wider
// than common phone viewports. It's currently hidden behind a hardcoded
// `skeleton` flag ("Coming Soon"), but the markup must already be wrapped
// for horizontal scrolling on mobile so the page never has to scroll
// sideways once the feature ships.

describe("AnnounceComponent mobile responsiveness", () => {
  it("wraps the announcements table in a horizontally scrollable container", () => {
    const dom = parseComponentTemplate(rawSource);
    const table = dom.querySelector("table.table");
    expect(table).not.toBeNull();
    expect(table.closest(".overflow-x-auto")).not.toBeNull();
  });
});
