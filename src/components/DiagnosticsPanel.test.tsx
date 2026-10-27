import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { DiagnosticsPanel } from "./DriveDashboard";
vi.mock("../lib/api", () => ({
  api: vi.fn().mockResolvedValue({
    report: "Shareable fixed status",
    details: "private-file-name.txt",
  }),
  asError: (e: unknown) => e,
}));
afterEach(cleanup);
it("copies only the status report and keeps local events collapsed with a sharing warning", async () => {
  const user = userEvent.setup();
  const writeText = vi
    .spyOn(navigator.clipboard, "writeText")
    .mockResolvedValue();
  render(<DiagnosticsPanel id="fixture" onError={vi.fn()} />);
  expect(
    await screen.findByDisplayValue("Shareable fixed status"),
  ).toBeInTheDocument();
  const details = screen
    .getByLabelText("Local connection events")
    .closest("details");
  expect(details).not.toHaveAttribute("open");
  expect(
    screen.getByText(/may still contain private file names/),
  ).toBeInTheDocument();
  await user.click(
    screen.getByRole("button", { name: "Copy shareable report" }),
  );
  expect(writeText).toHaveBeenCalledExactlyOnceWith("Shareable fixed status");
});
