import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import VerifyComponent from "./VerifyComponent.vue";
import { Guild, Vote, Verify, VerifyRole } from "../../stores/guild.js";
import { GuildMeta } from "../../stores/meta.js";

// Role changes are now asynchronous on the backend: the API answers 202 with
// a { job } snapshot and the component must render sync progress and poll
// until the reconciliation job completes (then refresh member counts).

vi.mock("../../helpers/verify.js", () => ({
  putVerifyRole: vi.fn(),
  deleteVerifyRole: vi.fn(),
  getVerifyJob: vi.fn(),
}));

vi.mock("../../stores/user.js", () => ({
  User: {
    loadCache: () => ({ token: { accessToken: "test-token" } }),
  },
}));

import { deleteVerifyRole, getVerifyJob } from "../../helpers/verify.js";

function runningJob(processed) {
  return { status: "running", total: 5000, processed, errors: 2 };
}

function mountWithOneRole() {
  const role = new VerifyRole("role-1", "Verified", "@example.com$", 3);
  const guild = new Guild("guild-1", new Verify([role], new Map()), new Vote([]));
  const guildMeta = new GuildMeta(
    "guild-1",
    "Test Guild",
    null,
    true,
    [{ id: "role-1", name: "Verified", color: 0 }],
    [],
  );
  return mount(VerifyComponent, {
    props: { guild, guildMeta },
    global: { stubs: { fa: true, RoleSelect: true, RoleTag: true } },
    attachTo: document.body,
  });
}

describe("VerifyComponent reconciliation job progress", () => {
  let modalTarget;

  beforeEach(() => {
    vi.useFakeTimers();
    modalTarget = document.createElement("div");
    modalTarget.id = "modal";
    document.body.appendChild(modalTarget);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
    modalTarget.remove();
  });

  it("shows sync progress after a role removal is accepted (202 + job)", async () => {
    deleteVerifyRole.mockResolvedValue({
      status: 202,
      data: { job: runningJob(100) },
    });

    const wrapper = mountWithOneRole();
    expect(wrapper.find('[data-testid="verify-job-progress"]').exists()).toBe(false);

    await wrapper.find("li.text-error").trigger("click");
    await flushPromises();

    const progress = wrapper.find('[data-testid="verify-job-progress"]');
    expect(progress.exists()).toBe(true);
    expect(progress.text()).toContain("Syncing roles…");
    expect(progress.text()).toContain("100");
    expect(progress.text()).toContain("5,000");
    // Skipped (403/404) members are surfaced, not hidden.
    expect(progress.text()).toContain("2 skipped");
    // The role row is gone from desired state immediately (202 semantics).
    expect(wrapper.props().guild.verify.roles).toHaveLength(0);

    wrapper.unmount();
  });

  it("polls the job endpoint and refreshes the guild once it succeeds", async () => {
    deleteVerifyRole.mockResolvedValue({
      status: 202,
      data: { job: runningJob(100) },
    });
    getVerifyJob
      .mockResolvedValueOnce({ data: runningJob(2600) })
      .mockResolvedValueOnce({
        data: { status: "succeeded", total: 5000, processed: 5000, errors: 2 },
      });

    const wrapper = mountWithOneRole();
    await wrapper.find("li.text-error").trigger("click");
    await flushPromises();

    // Big-guild job (total > 2000): polls on the backed-off 15s cadence.
    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    expect(getVerifyJob).toHaveBeenCalledTimes(1);
    expect(wrapper.find('[data-testid="verify-job-progress"]').text()).toContain("2,600");
    expect(wrapper.emitted("update")).toBeUndefined();

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    expect(getVerifyJob).toHaveBeenCalledTimes(2);
    // Succeeded: bar is gone, guild refresh requested (member counts).
    expect(wrapper.find('[data-testid="verify-job-progress"]').exists()).toBe(false);
    expect(wrapper.emitted("update")).toHaveLength(1);

    wrapper.unmount();
  });

  it("stops polling when unmounted mid-job", async () => {
    deleteVerifyRole.mockResolvedValue({
      status: 202,
      data: { job: runningJob(100) },
    });

    const wrapper = mountWithOneRole();
    await wrapper.find("li.text-error").trigger("click");
    await flushPromises();

    wrapper.unmount();
    await vi.advanceTimersByTimeAsync(60000);
    expect(getVerifyJob).not.toHaveBeenCalled();
  });

  it("a 204 removal (role was not configured) tracks no job", async () => {
    deleteVerifyRole.mockResolvedValue({ status: 204, data: null });

    const wrapper = mountWithOneRole();
    await wrapper.find("li.text-error").trigger("click");
    await flushPromises();

    expect(wrapper.find('[data-testid="verify-job-progress"]').exists()).toBe(false);
    wrapper.unmount();
  });
});
