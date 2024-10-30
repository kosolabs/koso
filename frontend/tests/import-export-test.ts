import { expect, test, type Page } from "@playwright/test";
import {
  getKosoGraph,
  getKosoProjectId,
  setupNewProject,
  tearDown,
} from "./utils";
import type { Readable } from "stream";

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
    const downloadPromise = download(page);
    await page.getByRole("button", { name: "Export Project" }).click();
    const exportedProject = await downloadPromise;
    expect(exportedProject.filename).toContain("export");
    expect(exportedProject.data["project_id"]).toEqual(
      await getKosoProjectId(page),
    );
    expect(exportedProject.data["data"]).toEqual(await getKosoGraph(page));

    // Import the project
    await page.goto("/projects");
    await page.getByRole("button", { name: "Import Project" }).click();
    const fileChooserPromise = page.waitForEvent("filechooser");
    page.locator("#projectImportFileInput").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles({
      name: exportedProject.filename,
      mimeType: "json",
      buffer: exportedProject.dataBuf,
    });
    await page.getByRole("button", { name: "Import", exact: true }).click();
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

    // Export the newly imported project.
    const downloadPromise2 = download(page);
    await page.getByRole("button", { name: "Export Project" }).click();
    const exportedProject2 = await downloadPromise2;
    expect(exportedProject2.filename).toContain("export");
    expect(exportedProject2.data["project_id"]).toEqual(
      await getKosoProjectId(page),
    );
    expect(exportedProject2.data["data"]).toEqual(await getKosoGraph(page));
    expect(exportedProject2.data["data"]).toEqual(exportedProject.data["data"]);

    // Validate that a new task can be created
    await page.getByRole("button", { name: "Add Task" }).last().click();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("row", { name: "Task 4" })).toBeVisible();
  });
});

type DownloadedProjectExport = {
  filename: string;
  dataBuf: Buffer;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  data: any;
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
