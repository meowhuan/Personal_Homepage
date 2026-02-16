let initialized = false;
let navigating = false;

const LEAVE_CLASS = "page-leave";
const LEAVE_DELAY_MS = 260;

const isModifiedClick = (event) =>
  event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0;

const isHashOnlyNavigation = (url) =>
  url.origin === window.location.origin &&
  url.pathname === window.location.pathname &&
  url.search === window.location.search &&
  url.hash &&
  url.hash !== window.location.hash;

export const installPageTransition = () => {
  if (initialized || typeof window === "undefined") return;
  initialized = true;

  const onClick = (event) => {
    if (event.defaultPrevented || isModifiedClick(event)) return;
    const anchor = event.target.closest("a");
    if (!anchor) return;

    const href = anchor.getAttribute("href");
    if (!href || href.startsWith("#")) return;
    if (href.startsWith("mailto:") || href.startsWith("tel:") || href.startsWith("javascript:")) return;
    if (anchor.target && anchor.target !== "_self") return;
    if (anchor.hasAttribute("download")) return;

    let nextUrl;
    try {
      nextUrl = new URL(anchor.href, window.location.href);
    } catch {
      return;
    }

    if (nextUrl.origin !== window.location.origin) return;
    if (isHashOnlyNavigation(nextUrl)) return;
    if (nextUrl.href === window.location.href) return;
    if (navigating) {
      event.preventDefault();
      return;
    }

    navigating = true;
    event.preventDefault();
    document.documentElement.classList.add(LEAVE_CLASS);
    window.setTimeout(() => {
      window.location.href = nextUrl.href;
    }, LEAVE_DELAY_MS);
  };

  const onPageShow = () => {
    navigating = false;
    document.documentElement.classList.remove(LEAVE_CLASS);
  };

  document.addEventListener("click", onClick, true);
  window.addEventListener("pageshow", onPageShow);
};
