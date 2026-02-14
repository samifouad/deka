export function renderWorkbench(container = document.body) {
  const filesIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 2h7a1.5 1.5 0 0 1 1.5 1.5V9A1.5 1.5 0 0 1 9.5 10.5h-7A1.5 1.5 0 0 1 1 9V3.5A1.5 1.5 0 0 1 2.5 2zm4 3h7A1.5 1.5 0 0 1 15 6.5V12a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 5 12V6.5A1.5 1.5 0 0 1 6.5 5z"></path></svg>';
  const playIcon =
    '<svg class="iconSvg" viewBox="0 0 16 16" aria-hidden="true"><path d="M4 2.8c0-.5.54-.8.97-.54l8.2 4.7a.62.62 0 0 1 0 1.08l-8.2 4.7A.62.62 0 0 1 4 12.2z"></path></svg>';

  container.innerHTML = `
    <main class="appShell">
      <aside class="activityBar" aria-label="Activity">
        <button id="treeBtn" class="activityBtn" type="button" title="Explorer">${filesIcon}</button>
        <button id="runBtn" class="activityBtn activityRun" type="button" title="Run">${playIcon}</button>
      </aside>

      <aside id="fileTree" class="explorerPane" aria-label="Explorer">
        <div class="explorerHeader">
          <span class="explorerTitle">Explorer</span>
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
              <div id="log" class="log"></div>
              <form id="termForm" class="termInputBar">
                <span id="termPrompt" class="termPrompt">/ $</span>
                <input id="termInput" class="termInput" type="text" autocomplete="off" />
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
  `
}
