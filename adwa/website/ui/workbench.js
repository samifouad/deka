export function renderWorkbench(container = document.body) {
  const filesIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 2h7a1.5 1.5 0 0 1 1.5 1.5V9A1.5 1.5 0 0 1 9.5 10.5h-7A1.5 1.5 0 0 1 1 9V3.5A1.5 1.5 0 0 1 2.5 2zm4 3h7A1.5 1.5 0 0 1 15 6.5V12a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 5 12V6.5A1.5 1.5 0 0 1 6.5 5z"></path></svg>';
  const playIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M4 2.8c0-.5.54-.8.97-.54l8.2 4.7a.62.62 0 0 1 0 1.08l-8.2 4.7A.62.62 0 0 1 4 12.2z"></path></svg>';
  const dbIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M8 1.5c-3.1 0-5.5 1.1-5.5 2.5v8c0 1.4 2.4 2.5 5.5 2.5s5.5-1.1 5.5-2.5V4c0-1.4-2.4-2.5-5.5-2.5zm0 1c2.8 0 4.5 1 4.5 1.5S10.8 5.5 8 5.5 3.5 4.5 3.5 4 5.2 2.5 8 2.5zm0 3c2.1 0 3.9-.5 4.5-1.2V6c0 .5-1.7 1.5-4.5 1.5S3.5 6.5 3.5 6V4.3c.6.7 2.4 1.2 4.5 1.2zm0 3c2.1 0 3.9-.5 4.5-1.2V9c0 .5-1.7 1.5-4.5 1.5S3.5 9.5 3.5 9V7.3c.6.7 2.4 1.2 4.5 1.2zm0 3c2.1 0 3.9-.5 4.5-1.2V12c0 .5-1.7 1.5-4.5 1.5S3.5 12.5 3.5 12v-1.7c.6.7 2.4 1.2 4.5 1.2z"></path></svg>';
  const settingsIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M9.7 1.2 10 2.6c.3.1.6.3.9.5l1.3-.6 1 1.7-1.1.9c.1.3.1.6.1.9s0 .6-.1.9l1.1.9-1 1.7-1.3-.6c-.3.2-.6.4-.9.5l-.3 1.4H7.9l-.3-1.4c-.3-.1-.6-.3-.9-.5l-1.3.6-1-1.7 1.1-.9A3.6 3.6 0 0 1 5.4 7c0-.3 0-.6.1-.9l-1.1-.9 1-1.7 1.3.6c.3-.2.6-.4.9-.5l.3-1.4h1.8ZM8.8 8a1.3 1.3 0 1 0-2.6 0 1.3 1.3 0 0 0 2.6 0Z"></path></svg>';

  container.innerHTML = `
    <main class="appShell">
      <aside class="activityBar" aria-label="Activity">
        <button id="treeBtn" class="activityBtn" type="button" title="Explorer">${filesIcon}</button>
        <button id="runBtn" class="activityBtn activityRun" type="button" title="Run">${playIcon}</button>
        <button id="dataBtn" class="activityBtn" type="button" title="Data">${dbIcon}</button>
        <button id="settingsBtn" class="activityBtn" type="button" title="Settings">${settingsIcon}</button>
      </aside>

      <aside id="fileTree" class="explorerPane" aria-label="Explorer">
        <div class="explorerHeader">
          <div class="explorerHeaderText">
            <span class="explorerTitle">Explorer</span>
            <span id="explorerPath" class="explorerPath">/home/user/demo</span>
          </div>
          <div class="explorerActions">
            <button id="newFileBtn" class="explorerActionBtn" type="button" title="New File">New</button>
            <button id="newFolderBtn" class="explorerActionBtn" type="button" title="New Folder">Folder</button>
          </div>
        </div>
        <ul id="fileTreeList" class="fileTreeList"></ul>
      </aside>
      <div id="splitExplorer" class="splitter splitterExplorer" role="separator" aria-orientation="vertical" aria-label="Resize explorer"></div>

      <section class="workspacePane">
        <section class="pane editorPane" aria-label="Editor">
          <div class="editorTabs" id="editorTabs" role="tablist" aria-label="Open files"></div>
          <div class="editorWrap">
            <div id="editor" class="editorHost"></div>
          </div>
          <textarea id="source" style="display:none"></textarea>
        </section>
        <div id="splitX" class="splitter splitterX" role="separator" aria-orientation="vertical" aria-label="Resize editor and preview"></div>

        <section id="rightPane" class="rightPane">
          <section class="pane previewPane" aria-label="Preview">
            <div class="previewBar">
              <input id="previewInput" class="previewInput" type="text" value="/" spellcheck="false" />
              <button id="previewGo" class="previewGo" type="button">Go</button>
              <span id="previewStatus" class="previewStatus">run mode</span>
            </div>
            <div class="resultWrap">
              <div id="resultBanner" class="resultBanner" role="status" aria-live="polite"></div>
              <iframe id="resultFrame" title="result"></iframe>
            </div>
          </section>
          <div id="splitY" class="splitter splitterY" role="separator" aria-orientation="horizontal" aria-label="Resize preview and terminal"></div>

          <section id="terminalPane" class="pane terminalPane" aria-label="Terminal">
            <div class="termWrap">
              <div class="termHeader">
                <div id="termTabs" class="termTabs"></div>
                <div class="termHeaderActions">
                  <button id="termNewTabBtn" class="termNewTabBtn" type="button" title="New terminal">+</button>
                  <button id="termMinimizeBtn" class="termMinimizeBtn" type="button" title="Minimize terminal">▾</button>
                </div>
              </div>
              <div id="log" class="log"></div>
              <div id="termBlockedLine" class="termBlockedLine hidden" aria-hidden="true">
                <span class="termBlockedCursor">█</span>
              </div>
              <form id="termForm" class="termInputBar">
                <span id="termPrompt" class="termPrompt">/ $</span>
                <input id="termInput" class="termInput" type="text" autocomplete="off" />
              </form>
              <form id="termStdinForm" class="termInputBar termInputBarStdin hidden" aria-label="Foreground stdin">
                <span id="termStdinPrompt" class="termPrompt">stdin &gt;</span>
                <input id="termStdinInput" class="termInput" type="text" autocomplete="off" />
              </form>
            </div>
          </section>
        </section>
      </section>
    </main>

    <aside id="helpModal" class="helpModal">
      <div class="helpCard">
        <div class="helpHeader">Keyboard Shortcuts</div>
        <ul class="helpBody">
          <li><kbd>Cmd/Ctrl</kbd> + <kbd>B</kbd> Toggle explorer</li>
          <li><kbd>Cmd/Ctrl</kbd> + <kbd>J</kbd> Toggle terminal</li>
          <li><kbd>Cmd/Ctrl</kbd> + <kbd>K</kbd> Focus editor</li>
          <li><kbd>Cmd/Ctrl</kbd> + <kbd>H</kbd> Toggle help</li>
          <li><kbd>Esc</kbd> Close overlays</li>
        </ul>
      </div>
    </aside>

    <aside id="dataModal" class="dataModal">
      <div class="dataCard">
        <div class="dataHeader">
          <strong>Data Explorer</strong>
          <button id="dataCloseBtn" class="dataCloseBtn" type="button" aria-label="Close">×</button>
        </div>
        <div class="dataForm">
          <label>DB <input id="dbNameInput" type="text" value="adwa_demo" /></label>
          <label>Table <input id="dbTableInput" type="text" value="packages" /></label>
          <label>ID <input id="dbIdInput" type="text" placeholder="1" /></label>
          <label>Name <input id="dbNameValueInput" type="text" placeholder="alpha" /></label>
          <label>Version <input id="dbVersionInput" type="text" placeholder="0.1.0" /></label>
        </div>
        <div class="dataActions">
          <button id="dbCreateBtn" type="button">Create DB</button>
          <button id="dbEnsureBtn" type="button">Ensure Table</button>
          <button id="dbRefreshBtn" type="button">Refresh</button>
          <button id="dbInsertBtn" type="button">Insert</button>
          <button id="dbUpdateBtn" type="button">Update</button>
          <button id="dbDeleteBtn" type="button">Delete</button>
          <button id="dbClearBtn" type="button">Clear</button>
        </div>
        <div class="dataSqlWrap">
          <textarea id="dbSqlInput" class="dataSqlInput" spellcheck="false" placeholder="select id, name, version from packages order by id asc limit 20"></textarea>
          <button id="dbRunSqlBtn" type="button">Run SQL</button>
        </div>
        <div id="dbStatus" class="dataStatus">ready</div>
        <div class="dataTableWrap">
          <table class="dataTable">
            <thead><tr id="dbColsRow"><th>id</th><th>name</th><th>version</th></tr></thead>
            <tbody id="dbRowsBody"></tbody>
          </table>
        </div>
      </div>
    </aside>

    <aside id="settingsModal" class="settingsModal">
      <div class="settingsCard">
        <div class="settingsHeader">
          <strong>Browser Storage</strong>
          <button id="settingsCloseBtn" class="settingsCloseBtn" type="button" aria-label="Close">×</button>
        </div>
        <div class="settingsBody">
          <p id="settingsSummary" class="settingsSummary">Loading…</p>
          <ul id="settingsList" class="settingsList"></ul>
        </div>
        <div class="settingsActions">
          <button id="settingsRefreshBtn" type="button">Refresh</button>
          <button id="settingsResetDbBtn" type="button">Reset DB</button>
          <button id="settingsResetFsBtn" type="button">Reset FS Snapshot</button>
          <button id="settingsResetAllBtn" type="button">Reset All</button>
        </div>
      </div>
    </aside>
  `
}

// Data modal is rendered by main.js and controlled via #dataBtn.
