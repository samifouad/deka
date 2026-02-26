export function createRoot(config = {}) {
  const root = {
    container: config.container || "#app",
    layout: config.layout || "",
  };

  function getContainer(selector) {
    if (!selector) {
      return null;
    }
    if (selector instanceof Element) {
      return selector;
    }
    return document.querySelector(selector);
  }

  function swapHtml(container, html) {
    if (!container) {
      return;
    }
    container.innerHTML = html;
  }

  const getLayout = (container) => {
    if (!container) return "";
    return container.dataset.layout || container.dataset.componentLayout || "";
  };
  const ensureLayout = (container) => {
    if (!container) return;
    if (root.layout && !getLayout(container)) {
      container.dataset.layout = root.layout;
    }
  };

  function applyPartial(payload, containerSelector) {
    if (!payload || typeof payload.html !== "string") {
      return;
    }
    const container = getContainer(containerSelector || root.container);
    swapHtml(container, payload.html);
    if (typeof payload.title === "string" && payload.title !== "") {
      document.title = payload.title;
    }
    if (typeof payload.head === "string" && payload.head !== "") {
      document.head.insertAdjacentHTML("beforeend", payload.head);
    }
  }

  async function fetchPartial(url) {
    const res = await fetch(url, {
      headers: { Accept: "text/x-phpx-fragment" },
      credentials: "same-origin",
    });
    if (!res.ok) {
      return null;
    }
    return res.json();
  }

  async function navigate(url, opts = {}) {
    const container = getContainer(opts.target || root.container);
    if (!container) {
      window.location.assign(url);
      return;
    }
    ensureLayout(container);
    const effectiveLayout = opts.layout || "";
    if (!effectiveLayout) {
      window.location.assign(url);
      return;
    }
    const currentLayout = getLayout(container);
    if (currentLayout !== effectiveLayout) {
      window.location.assign(url);
      return;
    }
    const payload = await fetchPartial(url);
    if (!payload) {
      window.location.assign(url);
      return;
    }
    applyPartial(payload, opts.target);
    if (opts.replace) {
      history.replaceState({ url }, "", url);
    } else {
      history.pushState({ url }, "", url);
    }
  }

  function onClick(event) {
    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }
    const link = target.closest("a[data-component-link]");
    if (!link) {
      return;
    }
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
      return;
    }
    event.preventDefault();
    const href = link.getAttribute("href");
    if (!href) {
      return;
    }
    const targetSel = link.getAttribute("data-component-target");
    const replace = link.hasAttribute("data-component-replace");
    const layout =
      link.getAttribute("data-layout") ||
      link.getAttribute("data-component-layout") ||
      root.layout ||
      "";
    navigate(href, { target: targetSel, replace, layout });
  }

  function onPopState(event) {
    const state = event.state;
    const url = state && state.url ? state.url : window.location.pathname;
    navigate(url, { replace: true });
  }

  const initialContainer = getContainer(root.container);
  ensureLayout(initialContainer);

  document.addEventListener("click", onClick);
  window.addEventListener("popstate", onPopState);

  return {
    navigate,
  };
}
