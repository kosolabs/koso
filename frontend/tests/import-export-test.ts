import type { ProjectExport } from "$lib/projects";
import { expect, test, type Page } from "@playwright/test";
import type { Readable } from "stream";
import {
  getKosoGraph,
  getKosoProjectId,
  setupNewProject,
  tearDown,
} from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("import export tests", () => {
  test.beforeEach(async ({ page }) => {
    await setupNewProject(page);
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test("export and import a project", async ({ page }) => {
    // Insert a few tasks.
    await page.getByRole("button", { name: "Add Task" }).last().click();
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();
    await page.keyboard.press("Escape");
    await page.getByRole("button", { name: "Insert" }).last().click();
    await page.keyboard.type("Task 2 name");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();
    await page.getByRole("button", { name: "Insert" }).click();
    await page.keyboard.type("Task 3 child name");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();

    // Export the project
    const downloadPromise = download(page);
    await page.getByRole("button", { name: "Export Project" }).click();
    const projectExport1 = await downloadPromise;
    expect(projectExport1.filename).toContain("export");
    expect(projectExport1.data.projectId).toEqual(await getKosoProjectId(page));
    expect(projectExport1.data.graph).toEqual(await getKosoGraph(page));

    // Import the project
    await page.goto("/projects");
    const fileChooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Import" }).click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles({
      name: projectExport1.filename,
      mimeType: "json",
      buffer: projectExport1.dataBuf,
    });
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

    // Export the newly imported project.
    const downloadPromise2 = download(page);
    await page.getByRole("button", { name: "Export Project" }).click();
    const projectExport2 = await downloadPromise2;
    expect(projectExport2.filename).toContain("export");
    expect(projectExport2.data.projectId).toEqual(await getKosoProjectId(page));
    expect(projectExport2.data.graph).toEqual(await getKosoGraph(page));
    expect(projectExport2.data.graph).toEqual(projectExport1.data.graph);

    // Validate that a new task can be created
    await page.getByRole("button", { name: "Insert" }).last().click();
    await expect(page.getByRole("row", { name: "Task 4" })).toBeVisible();
    await expect(page.getByRole("row", { name: "Task 4" })).toBeFocused;
  });
});

type DownloadedProjectExport = {
  filename: string;
  dataBuf: Buffer;
  data: ProjectExport;
};

async function download(page: Page): Promise<DownloadedProjectExport> {
  const download = await page.waitForEvent("download");
  const readable = await download.createReadStream();
  const buf = await streamToBuffer(readable);
  return {
    filename: download.suggestedFilename(),
    dataBuf: buf,
    data: JSON.parse(new TextDecoder().decode(buf)),
  };
}

function streamToBuffer(stream: Readable): Promise<Buffer> {
  const chunks: Uint8Array[] = [];
  return new Promise((resolve, reject) => {
    stream.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    stream.on("error", (err) => reject(err));
    stream.on("end", () => resolve(Buffer.concat(chunks)));
  });
}
