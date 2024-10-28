import { expect, test } from "@playwright/test";
import {
  getKosoGraph,
  getKosoProjectId,
  setupNewProject,
  tearDown,
} from "./utils";
import type { Readable } from "stream";
import path from "path";

test.describe.configure({ mode: "parallel" });

test.describe("import and export tests", () => {
  test.beforeEach(async ({ page }) => {
    await setupNewProject(page);
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test.use({ acceptDownloads: true });

  test("export and import a project", async ({ page }) => {
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

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export Project" }).click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("export");
    const downloadPath = download.suggestedFilename();
    await download.saveAs(downloadPath);
    const readable = await download.createReadStream();
    const buf = await streamToBuffer(readable);
    const exportData = JSON.parse(new TextDecoder().decode(buf));
    expect(exportData["project_id"]).toEqual(await getKosoProjectId(page));
    const graph = exportData["data"];
    expect(graph).toBeTruthy();

    const actualGraph = await getKosoGraph(page);
    expect(graph).toEqual(actualGraph);

    await page.goto("/projects");

    const fileChooserPromise = page.waitForEvent("filechooser");
    page.locator("#fileInput").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(downloadPath);
    await page.getByRole("button", { name: "Import Project" }).click();

    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

    const downloadPromise2 = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export Project" }).click();
    const download2 = await downloadPromise2;
    expect(download2.suggestedFilename()).toContain("export");
    const downloadPath2 = download2.suggestedFilename();
    await download2.saveAs(downloadPath2);
    const readable2 = await download2.createReadStream();
    const buf2 = await streamToBuffer(readable2);
    const exportData2 = JSON.parse(new TextDecoder().decode(buf2));
    expect(exportData2["project_id"]).toEqual(await getKosoProjectId(page));
    const graph2 = exportData2["data"];
    expect(graph2).toBeTruthy();

    const actualGraph2 = await getKosoGraph(page);
    expect(graph2).toEqual(actualGraph2);
    expect(graph).toEqual(graph2);
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
