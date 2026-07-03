import { describe, it, expect } from "vitest";
import rawSource from "./StreamAlertComponent.vue?raw";
import { parseComponentTemplate } from "../../testUtils/templateDom.js";

// Same mobile-scrolling issue as AnnounceComponent: the stream alerts table
// (Channel/Type/Name/Last Live/actions columns) needs a horizontally
// scrollable wrapper so it doesn't force the whole page to scroll sideways
// on narrow viewports.

describe("StreamAlertComponent mobile responsiveness", () => {
  it("wraps the stream alerts table in a horizontally scrollable container", () => {
    const dom = parseComponentTemplate(rawSource);
    const table = dom.querySelector("table.table");
    expect(table).not.toBeNull();
    expect(table.closest(".overflow-x-auto")).not.toBeNull();
  });
});
