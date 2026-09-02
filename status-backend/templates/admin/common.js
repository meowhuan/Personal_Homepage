(() => {
  const rawFetch = window.fetch.bind(window);
  let loginVisible = false;

  const requestPath = (input) => {
    try {
      return new URL(typeof input === "string" ? input : input.url, window.location.href).pathname;
    } catch (_) {
      return "";
    }
  };

  const showLogin = (message = "请使用管理员邮箱登录", registration = false) => {
    const existing = document.getElementById("admin-login-overlay");
    if (existing) {
      if (!registration) return;
      existing.remove();
      loginVisible = false;
    }
    if (loginVisible) return;
    loginVisible = true;
    const overlay = document.createElement("div");
    overlay.id = "admin-login-overlay";
    overlay.className = "admin-login-overlay";
    overlay.innerHTML = `
      <form class="admin-login-card">
        <div class="eyebrow">MEOW ADMIN</div>
        <h1>${registration ? "注册管理员" : "管理员登录"}</h1>
        <p class="hint">${message}</p>
        <label>邮箱<input name="email" type="email" autocomplete="username" required placeholder="admin@example.com" /></label>
        <label>密码<input name="password" type="password" autocomplete="new-password" required placeholder="至少 8 位" /></label>
        <button type="submit">${registration ? "完成注册并进入后台" : "登录后台"}</button>
        <div class="admin-login-error" aria-live="polite"></div>
      </form>`;
    document.body.appendChild(overlay);
    const form = overlay.querySelector("form");
    const error = overlay.querySelector(".admin-login-error");
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      const button = form.querySelector("button");
      button.disabled = true;
      error.textContent = "登录中…";
      try {
        const response = await rawFetch(registration ? "/admin/register" : "/admin/login", {
          method: "POST",
          credentials: "include",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            email: form.elements.email.value,
            password: form.elements.password.value
          })
        });
        const data = await response.json().catch(() => ({}));
        if (!response.ok) throw new Error(data.error || (registration ? "注册失败" : "邮箱或密码错误"));
        window.location.reload();
      } catch (err) {
        error.textContent = err.message || "登录失败";
        button.disabled = false;
      }
    });
    form.elements.email.focus();
  };

  window.fetch = async (input, init = {}) => {
    const response = await rawFetch(input, { ...init, credentials: init.credentials || "include" });
    const path = requestPath(input);
    if (response.status === 401 && path !== "/admin/session" && path !== "/admin/login") {
      showLogin("登录状态已失效，请重新登录");
    }
    return response;
  };

  const addNavigation = () => {
    const titlebar = document.querySelector(".titlebar");
    if (!titlebar || document.querySelector(".admin-nav")) return;
    const nav = document.createElement("nav");
    nav.className = "admin-nav";
    nav.innerHTML = `
      <div class="admin-nav-links">
        <a href="/admin">后台首页</a>
        <a href="/status/admin">状态</a>
        <a href="/schedule/admin">行程</a>
        <a href="/blog/admin">博客</a>
        <a href="/links/admin">友链</a>
      </div>
      <button class="admin-logout ghost" type="button">退出登录</button>`;
    titlebar.insertAdjacentElement("afterend", nav);
    nav.querySelector(".admin-logout").addEventListener("click", async () => {
      await rawFetch("/admin/logout", { method: "POST", credentials: "include" });
      showLogin("已退出，请重新登录");
    });
  };

  const checkSession = async () => {
    try {
      const response = await rawFetch("/admin/session", { credentials: "include" });
      const data = await response.json();
      if (!data.authenticated) {
        showLogin(
          data.registered ? "请使用管理员邮箱登录" : "首次部署，请先注册管理员账户",
          data.registered === false
        );
      }
    } catch (_) {
      showLogin("无法确认登录状态");
    }
  };

  document.addEventListener("DOMContentLoaded", () => {
    addNavigation();
    checkSession();
  });
})();
