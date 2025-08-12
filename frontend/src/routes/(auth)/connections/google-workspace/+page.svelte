<script lang="ts">
  import { headers } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import {
      CheckCircle,
      ExternalLink,
      FileText,
      Presentation,
      RefreshCw,
      Table,
      XCircle
  } from "@lucide/svelte";
  import { onMount } from "svelte";

  const auth = getAuthContext();

  interface GoogleDocument {
    id: string;
    name: string;
    mimeType: string;
    webViewLink: string;
    createdTime: string;
    modifiedTime: string;
    owners: Array<{ email: string; name: string }>;
    writers: Array<{ email: string; name: string }>;
    readers: Array<{ email: string; name: string }>;
  }

  interface DiscoverResponse {
    documents: GoogleDocument[];
    connectedDocumentIds: string[];
  }

  let loading = $state(false);
  let documents = $state<GoogleDocument[]>([]);
  let connectedDocuments = $state<GoogleDocument[]>([]);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);
  let selectedProjectId = $state<string>("");
  let projects = $state<Array<{ project_id: string; name: string }>>([]);

  onMount(async () => {
    await loadProjects();
  });

  async function loadProjects() {
    try {
      const response = await fetch("/api/projects", {
        headers: headers(auth),
      });
      if (response.ok) {
        projects = await response.json();
        if (projects.length > 0) {
          selectedProjectId = projects[0].project_id;
        }
      }
    } catch (err) {
      console.error("Failed to load projects:", err);
    }
  }

  async function discoverDocuments() {
    if (!selectedProjectId) {
      error = "Please select a project first";
      return;
    }

    loading = true;
    error = null;
    success = null;

    try {
      const response = await fetch(
        `/plugins/google-workspace/documents?project_id=${selectedProjectId}`,
        {
          headers: headers(auth),
        }
      );

      if (response.ok) {
        const data: DiscoverResponse = await response.json();
        documents = data.documents;
        
        // Filter connected documents
        connectedDocuments = documents.filter(doc => 
          data.connectedDocumentIds.includes(doc.id)
        );
        
        success = `Found ${documents.length} documents, ${connectedDocuments.length} already connected`;
      } else {
        error = "Failed to discover documents. Please ensure you have connected your Google Workspace account.";
      }
    } catch (err) {
      error = "Failed to discover documents";
      console.error(err);
    } finally {
      loading = false;
    }
  }

  async function connectDocument(documentId: string, documentType: string) {
    try {
      const response = await fetch(
        "/plugins/google-workspace/documents/connect",
        {
          method: "POST",
          headers: {
            ...headers(auth),
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            project_id: selectedProjectId,
            document_id: documentId,
            document_type: documentType,
          }),
        }
      );

      if (response.ok) {
        success = "Document connected successfully!";
        await discoverDocuments(); // Refresh the list
      } else {
        error = "Failed to connect document";
      }
    } catch (err) {
      error = "Failed to connect document";
      console.error(err);
    }
  }

  async function disconnectDocument(documentId: string) {
    try {
      const response = await fetch(
        "/plugins/google-workspace/documents/disconnect",
        {
          method: "DELETE",
          headers: {
            ...headers(auth),
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            project_id: selectedProjectId,
            document_id: documentId,
          }),
        }
      );

      if (response.ok) {
        success = "Document disconnected successfully!";
        await discoverDocuments(); // Refresh the list
      } else {
        error = "Failed to disconnect document";
      }
    } catch (err) {
      error = "Failed to disconnect document";
      console.error(err);
    }
  }

  async function syncDocument(documentId: string) {
    try {
      const response = await fetch(
        "/plugins/google-workspace/documents/sync",
        {
          method: "POST",
          headers: {
            ...headers(auth),
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            project_id: selectedProjectId,
            document_id: documentId,
          }),
        }
      );

      if (response.ok) {
        success = "Document sync completed!";
      } else {
        error = "Failed to sync document";
      }
    } catch (err) {
      error = "Failed to sync document";
      console.error(err);
    }
  }

  function getDocumentIcon(mimeType: string) {
    switch (mimeType) {
      case "application/vnd.google-apps.document":
        return FileText;
      case "application/vnd.google-apps.spreadsheet":
        return Table;
      case "application/vnd.google-apps.presentation":
        return Presentation;
      default:
        return FileText;
    }
  }

  function getDocumentType(mimeType: string) {
    switch (mimeType) {
      case "application/vnd.google-apps.document":
        return "Google Doc";
      case "application/vnd.google-apps.spreadsheet":
        return "Google Sheet";
      case "application/vnd.google-apps.presentation":
        return "Google Slides";
      default:
        return "Document";
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }

  function isConnected(documentId: string) {
    return connectedDocuments.some(doc => doc.id === documentId);
  }
</script>

<svelte:head>
  <title>Google Workspace - Koso</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 max-w-6xl">
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">Google Workspace Integration</h1>
    <p class="text-gray-600">
      Connect your Google Workspace documents to automatically track comments and required reviewers as Koso tasks.
    </p>
  </div>

  <!-- Project Selection -->
  <div class="bg-white rounded-lg shadow p-6 mb-6">
    <h2 class="text-xl font-semibold mb-4">Select Project</h2>
    <div class="flex gap-4 items-center">
      <select
        bind:value={selectedProjectId}
        class="flex-1 p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
      >
        {#each projects as project}
          <option value={project.project_id}>{project.name}</option>
        {/each}
      </select>
      <button
        on:click={discoverDocuments}
        disabled={loading || !selectedProjectId}
        class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <RefreshCw class="w-4 h-4" />
        Discover Documents
      </button>
    </div>
  </div>

  <!-- Status Messages -->
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
      <div class="flex">
        <XCircle class="w-5 h-5 text-red-400" />
        <div class="ml-3">
          <p class="text-sm text-red-800">{error}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if success}
    <div class="bg-green-50 border border-green-200 rounded-md p-4 mb-6">
      <div class="flex">
        <CheckCircle class="w-5 h-5 text-green-400" />
        <div class="ml-3">
          <p class="text-sm text-green-800">{success}</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Connected Documents -->
  {#if connectedDocuments.length > 0}
    <div class="bg-white rounded-lg shadow p-6 mb-6">
      <h2 class="text-xl font-semibold mb-4">Connected Documents</h2>
      <div class="space-y-4">
        {#each connectedDocuments as doc}
          <div class="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
            <div class="flex items-center gap-3">
              <svelte:component this={getDocumentIcon(doc.mimeType)} class="w-5 h-5 text-blue-600" />
              <div>
                <h3 class="font-medium text-gray-900">{doc.name}</h3>
                <p class="text-sm text-gray-500">{getDocumentType(doc.mimeType)} • Modified {formatDate(doc.modifiedTime)}</p>
              </div>
            </div>
            <div class="flex gap-2">
              <button
                on:click={() => syncDocument(doc.id)}
                class="flex items-center gap-2 px-3 py-1 border border-gray-300 rounded text-sm hover:bg-gray-50"
              >
                <RefreshCw class="w-4 h-4" />
                Sync
              </button>
              <button
                on:click={() => disconnectDocument(doc.id)}
                class="px-3 py-1 border border-gray-300 rounded text-sm text-red-600 hover:text-red-700 hover:bg-red-50"
              >
                Disconnect
              </button>
              <a
                href={doc.webViewLink}
                target="_blank"
                rel="noopener noreferrer"
                class="p-2 text-gray-400 hover:text-gray-600"
              >
                <ExternalLink class="w-4 h-4" />
              </a>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Available Documents -->
  {#if documents.length > 0}
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-xl font-semibold mb-4">Available Documents</h2>
      <div class="space-y-4">
        {#each documents as doc}
          <div class="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
            <div class="flex items-center gap-3">
              <svelte:component this={getDocumentIcon(doc.mimeType)} class="w-5 h-5 text-gray-400" />
              <div>
                <h3 class="font-medium text-gray-900">{doc.name}</h3>
                <p class="text-sm text-gray-500">
                  {getDocumentType(doc.mimeType)} • Modified {formatDate(doc.modifiedTime)}
                </p>
                {#if doc.owners.length > 0}
                  <p class="text-xs text-gray-400">Owner: {doc.owners[0].name}</p>
                {/if}
              </div>
            </div>
            <div class="flex gap-2">
              {#if isConnected(doc.id)}
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  <CheckCircle class="w-3 h-3 mr-1" />
                  Connected
                </span>
              {:else}
                <button
                  on:click={() => connectDocument(doc.id, getDocumentType(doc.mimeType).toLowerCase().replace(' ', ''))}
                  class="flex items-center gap-2 px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
                >
                  <CheckCircle class="w-4 h-4" />
                  Connect
                </button>
              {/if}
              <a
                href={doc.webViewLink}
                target="_blank"
                rel="noopener noreferrer"
                class="p-2 text-gray-400 hover:text-gray-600"
              >
                <ExternalLink class="w-4 h-4" />
              </a>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Empty State -->
  {#if !loading && documents.length === 0 && connectedDocuments.length === 0}
    <div class="bg-white rounded-lg shadow p-12 text-center">
      <FileText class="w-12 h-12 text-gray-400 mx-auto mb-4" />
      <h3 class="text-lg font-medium text-gray-900 mb-2">No documents found</h3>
      <p class="text-gray-500 mb-4">
        Click "Discover Documents" to find Google Workspace documents you can connect to this project.
      </p>
      <button
        on:click={discoverDocuments}
        disabled={!selectedProjectId}
        class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed mx-auto"
      >
        <RefreshCw class="w-4 h-4" />
        Discover Documents
      </button>
    </div>
  {/if}

  <!-- Loading State -->
  {#if loading}
    <div class="bg-white rounded-lg shadow p-12 text-center">
      <RefreshCw class="w-8 h-8 text-blue-600 mx-auto mb-4 animate-spin" />
      <p class="text-gray-500">Discovering documents...</p>
    </div>
  {/if}
</div>
