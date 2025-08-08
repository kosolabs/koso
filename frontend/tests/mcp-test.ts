import { expect, request, test } from "@playwright/test";
import { setupNewProject, tearDown } from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("mcp tests", () => {
  test.beforeEach(async ({ page }) => {
    await setupNewProject(page);
  });

  test.afterAll(async () => {
    await tearDown();
  });

  type RegistrationResponse = {
    client_id: string;
    client_name: string;
    scope: string;
    token_endpoint_auth_method: string;
    redirect_uris: string[];
    grant_types: string[];
    response_types: string[];
    client_secret_expires_at: number;
    client_secret: string;
  };

  type TokenResponse = {
    token_type: string;
    expires_in: number;
    scope: string;
    access_token: string;
    refresh_token: string;
  };

  test("oauth flow with authenticated client", async ({ page }) => {
    const host = new URL(page.url())?.host;
    const apiContext = await request.newContext({});
    const redirectUri = `http://${host}/oauth/callback/debug`;
    const state = "state424242";
    const codeVerifier = "qkkIJx-brh0RjiBH3RJZeIiam66UuxHCZmsR7DxPuYB";
    const codeChallenge = "5IcmHkDZ7xfw54Bs1U7ejZxKRZ5pxo0z_d5mCDWRlkc";

    // Register the client.
    const clientRes = await apiContext.post("/oauth/register", {
      data: {
        client_name: "mcp-test-client",
        scope: "read_write",
        redirect_uris: [redirectUri, "http://localhost/foo"],
        token_endpoint_auth_method: "client_secret_basic",
        grant_types: ["authorization_code", "refresh_token", "foo"],
        response_types: ["code", "bar"],
      },
      headers: {
        "Content-Type": `application/json`,
      },
    });
    expect(clientRes.ok()).toBeTruthy();
    const client: RegistrationResponse = await clientRes.json();
    expect(client).toMatchObject({
      client_name: "mcp-test-client",
      scope: "read_write",
      redirect_uris: [redirectUri, "http://localhost/foo"],
      token_endpoint_auth_method: "client_secret_basic",
      grant_types: ["authorization_code", "refresh_token"],
      response_types: ["code"],
    });
    expect(client.client_secret).toBeTruthy();

    // Craft the authorize URL.
    const authorizeUrl = new URL(
      `http://${host}/connections/mcp/oauth/authorize`,
    );
    authorizeUrl.searchParams.append("response_type", "code");
    authorizeUrl.searchParams.append("client_id", client.client_id);
    authorizeUrl.searchParams.append("state", state);
    authorizeUrl.searchParams.append("code_challenge", codeChallenge);
    authorizeUrl.searchParams.append("code_challenge_method", "S256");
    authorizeUrl.searchParams.append("redirect_uri", redirectUri);
    authorizeUrl.searchParams.append("scope", "read_write");
    authorizeUrl.searchParams.append("resource", `http://${host}/api/mcp/sse`);

    // Cancel authorization.
    await page.goto(authorizeUrl.toString());
    await page.getByRole("button", { name: "Cancel" }).click();
    await page.waitForURL(`${redirectUri}?**`);
    const cancelUrl = new URL(page.url());
    expect(Array.from(cancelUrl.searchParams.entries())).toEqual(
      expect.arrayContaining([
        ["state", state],
        ["error", "access_denied"],
        ["error_description", "Authorization cancelled by user."],
      ]),
    );

    // Approve authorization.
    await page.goto(authorizeUrl.toString());
    await page.getByRole("button", { name: "Authorize" }).click();
    await page.waitForURL(`${redirectUri}?**`);
    const authorizedUrl = new URL(page.url());
    expect(Array.from(authorizedUrl.searchParams.entries())).toEqual(
      expect.arrayContaining([["state", state]]),
    );
    const code = authorizedUrl.searchParams.get("code") || "";
    expect(code).toBeTruthy();

    // Issue the access token.
    const tokenRes = await apiContext.post("/oauth/token", {
      form: {
        grant_type: "authorization_code",
        client_id: client.client_id,
        client_secret: client.client_secret,
        scope: "read_write",
        resource: `http://${host}/api/mcp/sse`,
        redirect_uri: redirectUri,
        code: code,
        code_verifier: codeVerifier,
      },
      headers: {
        "Content-Type": `application/x-www-form-urlencoded`,
      },
    });
    expect(tokenRes.ok()).toBeTruthy();
    const token: TokenResponse = await tokenRes.json();
    expect(token).toMatchObject({
      scope: "read_write",
      token_type: "Bearer",
    });
    expect(token.access_token).toBeTruthy();
    expect(token.refresh_token).toBeTruthy();
    expect(token.expires_in).toBeTruthy();

    // Test the token by sending an initialize request to the MCP server.
    const mcpRes = await apiContext.post("/api/mcp/sse", {
      data: {
        jsonrpc: "2.0",
        id: 0,
        method: "initialize",
        params: {
          protocolVersion: "2025-06-18",
          capabilities: {
            sampling: {},
            elicitation: {},
            roots: { listChanged: true },
          },
          clientInfo: { name: "mcp-test", version: "0.16.2" },
        },
      },
      headers: {
        Authorization: `Bearer ${token.access_token}`,
        Accept: "text/event-stream, application/json",
        "Content-Type": "application/json",
      },
    });
    expect(mcpRes.ok()).toBeTruthy();
    const mcpFrame = await mcpRes.text();
    expect(mcpFrame).toMatch(/^data: .+/);
    expect(JSON.parse(mcpFrame.substring(6))).toMatchObject({
      jsonrpc: "2.0",
      id: 0,
      result: {
        protocolVersion: "2025-03-26",
        capabilities: { prompts: {}, resources: {}, tools: {} },
        serverInfo: { name: "rmcp" },
        instructions: "This server provides access to Koso projects and tasks",
      },
    });

    // Refresh the token.
    const refreshRes = await apiContext.post("/oauth/token", {
      form: {
        grant_type: "refresh_token",
        client_id: client.client_id,
        client_secret: client.client_secret,
        refresh_token: token.refresh_token,
      },
      headers: {
        "Content-Type": `application/x-www-form-urlencoded`,
      },
    });
    expect(refreshRes.ok()).toBeTruthy();
    const refresh: TokenResponse = await refreshRes.json();
    expect(refresh).toMatchObject({
      scope: "read_write",
      token_type: "Bearer",
    });
    expect(refresh.access_token).toBeTruthy();
    expect(refresh.refresh_token).toBeTruthy();
    expect(refresh.expires_in).toBeTruthy();
  });

  test("oauth flow with unauthenticated client", async ({ page }) => {
    const host = new URL(page.url())?.host;
    const apiContext = await request.newContext({});
    const redirectUri = `http://${host}/oauth/callback/debug`;

    // Register the client.
    const clientRes = await apiContext.post("/oauth/register", {
      data: {
        client_name: "mcp-test-client",
        redirect_uris: [redirectUri],
        token_endpoint_auth_method: "none",
      },
      headers: {
        "Content-Type": `application/json`,
      },
    });
    expect(clientRes.ok()).toBeTruthy();
    const client: RegistrationResponse = await clientRes.json();
    expect(client).toMatchObject({
      client_name: "mcp-test-client",
      scope: "read_write",
      redirect_uris: [redirectUri],
      token_endpoint_auth_method: "none",
      grant_types: ["authorization_code", "refresh_token"],
      response_types: ["code"],
    });
    expect(client.client_secret).toBeTruthy();

    // Craft the authorize URL.
    const authorizeUrl = new URL(
      `http://${host}/connections/mcp/oauth/authorize`,
    );
    authorizeUrl.searchParams.append("response_type", "code");
    authorizeUrl.searchParams.append("client_id", client.client_id);
    authorizeUrl.searchParams.append("redirect_uri", redirectUri);

    // Approve authorization.
    await page.goto(authorizeUrl.toString());
    await page.getByRole("button", { name: "Authorize" }).click();
    await page.waitForURL(`${redirectUri}?**`);
    const authorizedUrl = new URL(page.url());
    expect(authorizeUrl.searchParams.has("state")).toBeFalsy();
    const code = authorizedUrl.searchParams.get("code") || "";
    expect(code).toBeTruthy();

    // Issue the access token.
    const tokenRes = await apiContext.post("/oauth/token", {
      form: {
        grant_type: "authorization_code",
        client_id: client.client_id,
        code: code,
      },
      headers: {
        "Content-Type": `application/x-www-form-urlencoded`,
      },
    });
    expect(tokenRes.ok()).toBeTruthy();
    const token: TokenResponse = await tokenRes.json();
    expect(token).toMatchObject({
      scope: "read_write",
      token_type: "Bearer",
    });
    expect(token.access_token).toBeTruthy();
    expect(token.refresh_token).toBeTruthy();
    expect(token.expires_in).toBeTruthy();

    // Test the token by sending an initialize request to the MCP server.
    const mcpRes = await apiContext.post("/api/mcp/sse", {
      data: {
        jsonrpc: "2.0",
        id: 0,
        method: "initialize",
        params: {
          protocolVersion: "2025-06-18",
          capabilities: {
            sampling: {},
            elicitation: {},
            roots: { listChanged: true },
          },
          clientInfo: { name: "mcp-test", version: "0.16.2" },
        },
      },
      headers: {
        Authorization: `Bearer ${token.access_token}`,
        Accept: "text/event-stream, application/json",
        "Content-Type": "application/json",
      },
    });
    expect(mcpRes.ok()).toBeTruthy();
    const mcpFrame = await mcpRes.text();
    expect(mcpFrame).toMatch(/^data: .+/);
    expect(JSON.parse(mcpFrame.substring(6))).toMatchObject({
      jsonrpc: "2.0",
      id: 0,
      result: {
        protocolVersion: "2025-03-26",
        capabilities: { prompts: {}, resources: {}, tools: {} },
        serverInfo: { name: "rmcp" },
        instructions: "This server provides access to Koso projects and tasks",
      },
    });

    // Refresh the token.
    const refreshRes = await apiContext.post("/oauth/token", {
      form: {
        grant_type: "refresh_token",
        client_id: client.client_id,
        refresh_token: token.refresh_token,
      },
      headers: {
        "Content-Type": `application/x-www-form-urlencoded`,
      },
    });
    expect(refreshRes.ok()).toBeTruthy();
    const refresh: TokenResponse = await refreshRes.json();
    expect(refresh).toMatchObject({
      scope: "read_write",
      token_type: "Bearer",
    });
    expect(refresh.access_token).toBeTruthy();
    expect(refresh.refresh_token).toBeTruthy();
    expect(refresh.expires_in).toBeTruthy();
  });
});
