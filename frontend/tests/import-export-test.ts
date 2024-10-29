import { expect, test } from "@playwright/test";
import {
  getKosoGraph,
  getKosoProjectId,
  setupNewProject,
  tearDown,
} from "./utils";
import type { Readable } from "stream";

test.describe.configure({ mode: "parallel" });

test.describe("import and export tests", () => {
  test.beforeEach(async ({ page }) => {
    await setupNewProject(page);
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test("export and import a project", async ({ page }) => {
    // Insert a few tasks.
    await page.getByRole("button", { name: "Add Task" }).last().click();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();
    await page.getByRole("button", { name: "Add Task" }).last().click();
    await page.keyboard.type("Task 2 name");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();
    await page.getByRole("button", { name: "Add Child" }).click();
    await page.keyboard.type("Task 3 child name");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();

    // Export the project
    let downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export Project" }).click();
    let download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("export");
    let readable = await download.createReadStream();
    let buf = await streamToBuffer(readable);
    let exportData = JSON.parse(new TextDecoder().decode(buf));
    expect(exportData["project_id"]).toEqual(await getKosoProjectId(page));
    const graph = exportData["data"];
    expect(graph).toBeTruthy();
    expect(graph).toEqual(await getKosoGraph(page));

    // Import the project
    await page.goto("/projects");
    const fileChooserPromise = page.waitForEvent("filechooser");
    page.locator("#fileInput").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles({
      name: download.suggestedFilename(),
      mimeType: "json",
      buffer: buf,
    });
    await page.getByRole("button", { name: "Import Project" }).click();
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

    // Export the newly imported project.
    downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export Project" }).click();
    download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("export");
    readable = await download.createReadStream();
    buf = await streamToBuffer(readable);
    exportData = JSON.parse(new TextDecoder().decode(buf));
    expect(exportData["project_id"]).toEqual(await getKosoProjectId(page));
    expect(exportData["data"]).toEqual(await getKosoGraph(page));
    expect(exportData["data"]).toEqual(graph);
  });
});

function streamToBuffer(stream: Readable): Promise<Buffer> {
  const chunks: Uint8Array[] = [];
  return new Promise((resolve, reject) => {
    stream.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    stream.on("error", (err) => reject(err));
    stream.on("end", () => resolve(Buffer.concat(chunks)));
  });
}
