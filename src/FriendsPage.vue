<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";

const LINKS_API_BASE = "https://m.ratf.cn/links";
const APPLY_API_URL = "https://m.ratf.cn/links/apply";
const APPLY_CONFIG_URL = "https://m.ratf.cn/links/apply/config";
const VERIFY_HTTP_URL = "https://m.ratf.cn/links/verify/http";
const VERIFY_EMAIL_SEND_URL = "https://m.ratf.cn/links/verify/email/send";
const APPLY_CAPTCHA_CONTAINER_ID = "friend-link-captcha";
const EMAIL_CAPTCHA_CONTAINER_ID = "friend-link-email-captcha";

const isNight = ref(false);
const themeMedia = ref(null);
const loading = ref(false);
const error = ref(false);
const links = ref([]);
const submitLoading = ref(false);
const submitError = ref("");
const submitSuccess = ref("");
const copyStatus = ref("");
const verifyHint = reactive({
  application_id: null,
  verify_status: "",
  message: "",
  verify_token: "",
  verify_deadline: 0,
  actionLoading: false
});
const modal = reactive({
  open: false,
  title: "",
  message: ""
});
const applyConfig = reactive({
  captcha_enabled: false,
  captcha_provider: "",
  captcha_site_key: "",
  captcha_secret_set: true
});
const applyCaptchaWidgetId = ref(null);
const emailCaptchaWidgetId = ref(null);
const applyAltchaPayload = ref("");
const emailAltchaPayload = ref("");

const form = reactive({
  site_name: "",
  site_url: "",
  avatar_url: "",
  description: "",
  email: "",
  note: ""
});

const fallbackLinks = [
  {
    id: "fallback-1",
    name: "Meowhuan Blog",
    url: "https://meowra.cn",
    avatar_url: "/logo.png",
    description: "主页和博客内容站点。",
    tags: "个人主页"
  }
];

const normalizeUrl = (raw) => {
  const value = String(raw || "").trim();
  if (!value) return "";
  if (/^https?:\/\//i.test(value)) return value;
  return `https://${value}`;
};

const resetForm = () => {
  form.site_name = "";
  form.site_url = "";
  form.avatar_url = "";
  form.description = "";
  form.email = "";
  form.note = "";
};

const onSystemThemeChange = (event) => {
  if (localStorage.getItem("meow-theme")) return;
  isNight.value = event.matches;
};

const toggleTheme = () => {
  isNight.value = !isNight.value;
  localStorage.setItem("meow-theme", isNight.value ? "night" : "day");
  if (themeMedia.value) {
    themeMedia.value.removeEventListener("change", onSystemThemeChange);
    themeMedia.value = null;
  }
};

const fetchLinks = async () => {
  loading.value = true;
  try {
    const res = await fetch(LINKS_API_BASE);
    if (!res.ok) throw new Error("links fetch failed");
    const data = await res.json();
    links.value = Array.isArray(data) ? data : [];
    error.value = false;
  } catch {
    links.value = fallbackLinks;
    error.value = true;
  } finally {
    loading.value = false;
  }
};

const submitApply = async () => {
  submitSuccess.value = "";
  submitError.value = "";

  const siteName = form.site_name.trim();
  const siteUrl = normalizeUrl(form.site_url);
  const email = form.email.trim();
  if (!siteName || !siteUrl || !email) {
    submitError.value = "请填写站点名称、站点地址和联系邮箱。";
    return;
  }
  const blockedEmails = new Set([
    "meowhuan@qq.com",
    "meowhuan@meowra.cn",
    "3250315682@qq.com"
  ]);
  if (blockedEmails.has(email.toLowerCase())) {
    submitError.value = "该邮箱不支持用于友链申请。";
    return;
  }
  if (siteName.length > 32) {
    submitError.value = "站点名称最多 32 个字符。";
    return;
  }

  const captchaToken = getCaptchaToken("email");
  if (applyConfig.captcha_enabled && !captchaToken) {
    submitError.value = "请先完成人机验证。";
    return;
  }

  submitLoading.value = true;
  try {
    const res = await fetch(APPLY_API_URL, {
      method: "POST",
      headers: {
        "content-type": "application/json"
      },
      body: JSON.stringify({
        site_name: siteName,
        site_url: siteUrl,
        avatar_url: normalizeUrl(form.avatar_url),
        description: form.description.trim(),
        email,
        note: form.note.trim(),
        captcha_token: captchaToken || undefined
      })
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      throw new Error(data?.message || "提交失败，请稍后再试。");
    }
    submitSuccess.value = data?.message || "申请已提交，感谢喵喵投递。";
    verifyHint.application_id = Number(data?.application_id || 0) || null;
    verifyHint.verify_status = String(data?.verify_status || "");
    verifyHint.message = data?.message || "";
    verifyHint.verify_token = String(data?.verify_token || "");
    verifyHint.verify_deadline = Number(data?.verify_deadline || 0) || 0;
    resetForm();
    resetCaptcha("apply");
  } catch (err) {
    submitError.value = err instanceof Error ? err.message : "提交失败，请稍后再试。";
  } finally {
    submitLoading.value = false;
  }
};

const triggerHttpVerify = async () => {
  if (!verifyHint.application_id || verifyHint.actionLoading) return;
  verifyHint.actionLoading = true;
  submitError.value = "";
  try {
    const res = await fetch(VERIFY_HTTP_URL, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ application_id: verifyHint.application_id })
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(data?.message || "HTTP 验证失败");
    submitSuccess.value = data?.message || "HTTP 验证成功";
    verifyHint.message = submitSuccess.value;
    verifyHint.verify_status = "pending";
    openModal("验证成功", submitSuccess.value);
  } catch (err) {
    submitError.value = err instanceof Error ? err.message : "HTTP 验证失败";
    openModal("验证失败", submitError.value);
  } finally {
    verifyHint.actionLoading = false;
  }
};

const sendVerifyEmail = async () => {
  if (!verifyHint.application_id || verifyHint.actionLoading) return;
  verifyHint.actionLoading = true;
  submitError.value = "";
  const captchaToken = getCaptchaToken("apply");
  if (applyConfig.captcha_enabled && !captchaToken) {
    submitError.value = "请先完成人机验证。";
    verifyHint.actionLoading = false;
    openModal("发送失败", submitError.value);
    return;
  }
  try {
    const res = await fetch(VERIFY_EMAIL_SEND_URL, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        application_id: verifyHint.application_id,
        verify_token: verifyHint.verify_token || "",
        captcha_token: captchaToken || undefined
      })
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(data?.message || "发送验证邮件失败");
    submitSuccess.value = data?.message || "验证邮件已发送";
    verifyHint.message = submitSuccess.value;
    if (data?.verify_token) {
      verifyHint.verify_token = String(data.verify_token);
    }
    openModal("邮件已发送", submitSuccess.value);
  } catch (err) {
    submitError.value = err instanceof Error ? err.message : "发送验证邮件失败";
    openModal("发送失败", submitError.value);
  } finally {
    verifyHint.actionLoading = false;
    resetCaptcha("email");
  }
};

const openModal = (title, message) => {
  modal.title = title || "提示";
  modal.message = message || "";
  modal.open = true;
};

const closeModal = () => {
  modal.open = false;
};

const getCaptchaToken = (target) => {
  if (!applyConfig.captcha_enabled) return "";
  if (applyConfig.captcha_provider === "altcha") {
    return target === "email" ? emailAltchaPayload.value : applyAltchaPayload.value;
  }
  if (applyConfig.captcha_provider === "turnstile") {
    const widgetId = target === "email" ? emailCaptchaWidgetId.value : applyCaptchaWidgetId.value;
    return (window.turnstile?.getResponse?.(widgetId) || "").trim();
  }
  if (applyConfig.captcha_provider === "hcaptcha") {
    const widgetId = target === "email" ? emailCaptchaWidgetId.value : applyCaptchaWidgetId.value;
    return (window.hcaptcha?.getResponse?.(widgetId) || "").trim();
  }
  return "";
};

const resetCaptcha = (target) => {
  if (!applyConfig.captcha_enabled) return;
  if (applyConfig.captcha_provider === "altcha") {
    const clearPayload = (refValue) => {
      refValue.value = "";
    };
    if (target === "apply") {
      clearPayload(applyAltchaPayload);
      void renderCaptchaWidget(APPLY_CAPTCHA_CONTAINER_ID, applyCaptchaWidgetId);
    } else if (target === "email") {
      clearPayload(emailAltchaPayload);
      void renderCaptchaWidget(EMAIL_CAPTCHA_CONTAINER_ID, emailCaptchaWidgetId);
    } else {
      clearPayload(applyAltchaPayload);
      clearPayload(emailAltchaPayload);
      void renderCaptchaWidgets();
    }
    return;
  }
  const resetOne = (widgetId) => {
    if (widgetId == null) return;
    if (applyConfig.captcha_provider === "turnstile") {
      window.turnstile?.reset?.(widgetId);
    } else if (applyConfig.captcha_provider === "hcaptcha") {
      window.hcaptcha?.reset?.(widgetId);
    }
  };
  if (target === "apply") {
    resetOne(applyCaptchaWidgetId.value);
  } else if (target === "email") {
    resetOne(emailCaptchaWidgetId.value);
  } else {
    resetOne(applyCaptchaWidgetId.value);
    resetOne(emailCaptchaWidgetId.value);
  }
};

const ensureCaptchaScript = async (provider) => {
  if (!provider) return;
  const scriptId = provider === "turnstile"
    ? "captcha-turnstile-js"
    : provider === "hcaptcha"
      ? "captcha-hcaptcha-js"
      : "captcha-altcha-js";
  if (document.getElementById(scriptId)) return;
  const src = provider === "turnstile"
    ? "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit"
    : provider === "hcaptcha"
      ? "https://js.hcaptcha.com/1/api.js?render=explicit"
      : "https://cdn.jsdelivr.net/gh/altcha-org/altcha/dist/altcha.min.js";
  await new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.id = scriptId;
    script.src = src;
    if (provider === "altcha") {
      script.type = "module";
    }
    script.async = true;
    script.defer = true;
    script.onload = resolve;
    script.onerror = () => reject(new Error("captcha script load failed"));
    document.head.appendChild(script);
  });
};

const renderCaptchaWidget = async (containerId, widgetRef) => {
  if (!applyConfig.captcha_enabled || !applyConfig.captcha_provider || !applyConfig.captcha_site_key) return;
  const container = document.getElementById(containerId);
  if (!container) {
    widgetRef.value = null;
    return;
  }
  container.innerHTML = "";
  widgetRef.value = null;
  await ensureCaptchaScript(applyConfig.captcha_provider);
  if (applyConfig.captcha_provider === "turnstile" && window.turnstile?.render) {
    widgetRef.value = window.turnstile.render(`#${containerId}`, {
      sitekey: applyConfig.captcha_site_key,
      theme: isNight.value ? "dark" : "light"
    });
  } else if (applyConfig.captcha_provider === "hcaptcha" && window.hcaptcha?.render) {
    widgetRef.value = window.hcaptcha.render(containerId, {
      sitekey: applyConfig.captcha_site_key,
      theme: isNight.value ? "dark" : "light"
    });
  } else if (applyConfig.captcha_provider === "altcha") {
    const widget = document.createElement("altcha-widget");
    widget.setAttribute("challengeurl", applyConfig.captcha_site_key);
    widget.setAttribute("name", containerId === EMAIL_CAPTCHA_CONTAINER_ID ? "altcha-email" : "altcha-apply");
    widget.addEventListener("statechange", (event) => {
      const detail = event?.detail || {};
      const payload = typeof detail.payload === "string" ? detail.payload.trim() : "";
      if (payload) {
        if (containerId === EMAIL_CAPTCHA_CONTAINER_ID) {
          emailAltchaPayload.value = payload;
        } else {
          applyAltchaPayload.value = payload;
        }
      } else if (detail.state && detail.state !== "verified") {
        if (containerId === EMAIL_CAPTCHA_CONTAINER_ID) {
          emailAltchaPayload.value = "";
        } else {
          applyAltchaPayload.value = "";
        }
      }
    });
    container.appendChild(widget);
  }
};

const renderCaptchaWidgets = async () => {
  await renderCaptchaWidget(APPLY_CAPTCHA_CONTAINER_ID, applyCaptchaWidgetId);
  await renderCaptchaWidget(EMAIL_CAPTCHA_CONTAINER_ID, emailCaptchaWidgetId);
};

const loadApplyConfig = async () => {
  try {
    const res = await fetch(APPLY_CONFIG_URL, { cache: "no-store" });
    if (!res.ok) return;
    const cfg = await res.json();
    applyConfig.captcha_enabled = Boolean(cfg?.captcha_enabled);
    applyConfig.captcha_provider = String(cfg?.captcha_provider || "").trim();
    applyConfig.captcha_site_key = String(cfg?.captcha_site_key || "").trim();
    applyConfig.captcha_secret_set = cfg?.captcha_secret_set !== false;
    await renderCaptchaWidgets();
  } catch {
    applyConfig.captcha_enabled = false;
    applyConfig.captcha_provider = "";
    applyConfig.captcha_site_key = "";
    applyConfig.captcha_secret_set = true;
  }
};

const splitTags = (tagValue) =>
  String(tagValue || "")
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);

const shownLinks = computed(() => links.value);
const mySiteInfo = {
  site_name: "Meowhuan的个人主页",
  site_url: "https://www.meowra.cn/",
  description: "一只处于互联网边缘的小猫的个人主页",
  avatar_url: "https://www.meowra.cn/logo.png"
};
const mySiteOneClickText = [
  mySiteInfo.site_name,
  mySiteInfo.description,
  mySiteInfo.site_url,
  mySiteInfo.avatar_url
].join("\n");

const copyText = async (text) => {
  copyStatus.value = "";
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      const input = document.createElement("textarea");
      input.value = text;
      document.body.appendChild(input);
      input.select();
      document.execCommand("copy");
      document.body.removeChild(input);
    }
    copyStatus.value = "已复制";
    openModal("已复制", "内容已复制到剪贴板。");
    window.setTimeout(() => {
      copyStatus.value = "";
    }, 1400);
  } catch {
    copyStatus.value = "复制失败，请手动复制";
    openModal("复制失败", "请手动复制内容。");
  }
};

onMounted(() => {
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) {
    isNight.value = savedTheme === "night";
  } else if (window.matchMedia) {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    isNight.value = media.matches;
    themeMedia.value = media;
    media.addEventListener("change", onSystemThemeChange);
  }
  fetchLinks();
  loadApplyConfig();
});

watch(
  () => [applyConfig.captcha_enabled, applyConfig.captcha_provider, applyConfig.captcha_site_key, isNight.value],
  async () => {
    if (!applyConfig.captcha_enabled) return;
    await nextTick();
    await renderCaptchaWidgets();
  }
);

watch(
  () => verifyHint.application_id,
  async (value) => {
    if (!value || !applyConfig.captcha_enabled) return;
    await nextTick();
    await renderCaptchaWidget(EMAIL_CAPTCHA_CONTAINER_ID, emailCaptchaWidgetId);
  }
);

onBeforeUnmount(() => {
  if (themeMedia.value) {
    themeMedia.value.removeEventListener("change", onSystemThemeChange);
  }
});
</script>

<template>
  <div
    class="min-h-screen font-body transition-colors duration-700 ease-in-out meow-bg page-fade"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink meow-night'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink meow-day'"
  >
    <main class="relative mx-auto w-[min(1040px,92vw)] py-10">
      <button
        class="cord-switch"
        type="button"
        @click="toggleTheme"
        :class="isNight ? 'cord-switch-night' : 'cord-switch-day'"
        aria-label="切换深夜模式"
      >
        <span class="cord-line"></span>
        <span class="cord-knob">{{ isNight ? "🌙" : "☀️" }}</span>
        <span class="cord-label" aria-hidden="true"></span>
      </button>

      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 class="font-display text-3xl sm:text-4xl">喵喵友链</h1>
          <p class="mt-2 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
            收录友站，也欢迎提交友链申请。
          </p>
        </div>
        <div class="flex gap-2">
          <a
            href="/"
            class="meow-btn-ghost"
            :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
          >
            返回主页
          </a>
          <a
            href="/blog.html"
            class="meow-btn-ghost"
            :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
          >
            博客
          </a>
        </div>
      </div>

      <section class="mt-8 grid gap-4 md:grid-cols-[1.2fr_0.8fr]">
        <div class="space-y-4">
          <article
            class="meow-card motion-card rounded-3xl p-5"
            :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
          >
            <div class="flex items-center justify-between gap-3">
              <h2 class="font-display text-2xl">友站列表</h2>
              <button
                type="button"
                class="meow-pill motion-press"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : ''"
                @click="fetchLinks"
              >
                刷新
              </button>
            </div>
            <p v-if="loading" class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">加载中...</p>
            <p v-else-if="shownLinks.length === 0" class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">还没有收录友链。</p>
            <p v-if="error" class="mt-3 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">后端暂不可用，显示的是占位数据。</p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
              <a
                v-for="item in shownLinks"
                :key="item.id"
                :href="item.url"
                target="_blank"
                rel="noreferrer"
                class="friend-card motion-press"
                :class="isNight ? 'friend-card-night' : ''"
              >
                <img
                  :src="item.avatar_url || '/logo.png'"
                  :alt="`${item.name} avatar`"
                  class="h-11 w-11 rounded-full border object-cover"
                  :class="isNight ? 'border-meow-night-line' : 'border-meow-line'"
                />
                <div class="min-w-0 flex-1">
                  <div class="truncate text-sm font-700">{{ item.name }}</div>
                  <div class="mt-1 line-clamp-2 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                    {{ item.description || "这个站长很懒，还没写简介。" }}
                  </div>
                  <div class="mt-2 flex flex-wrap gap-1" v-if="splitTags(item.tags).length > 0">
                    <span v-for="tag in splitTags(item.tags)" :key="`${item.id}-${tag}`" class="meow-pill text-[10px]">
                      #{{ tag }}
                    </span>
                  </div>
                </div>
              </a>
            </div>
          </article>
        </div>

        <div class="space-y-4">
          <article
            class="meow-card motion-card rounded-3xl p-5"
            :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
          >
            <h2 class="font-display text-2xl">申请友链</h2>
            <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              请确保站点可访问，内容健康，且已添加本站后再提交申请。
            </p>
            <div
              class="mt-3 rounded-2xl border p-3 text-xs leading-relaxed"
              :class="isNight ? 'border-meow-night-line bg-meow-night-bg/40 text-meow-night-soft' : 'border-meow-line bg-white/60 text-meow-soft'"
            >
              <div class="font-700" :class="isNight ? 'text-meow-night-ink' : 'text-meow-ink'">验证声明</div>
              <div class="mt-1">申请提交后不会立即进入审核，需先完成以下任一验证：</div>
              <div>1. 邮箱验证（推荐）：向申请邮箱发送验证链接并点击确认。</div>
              <div>2. HTTP 验证：在站点 `/.well-known/meow-links.txt` 写入 token；</div>
              <div>3. DNS 验证：添加 TXT 记录 `_meow-links`，值包含 token；</div>
              <div>4. Meta 验证：首页 `&lt;head&gt;` 加入 `&lt;meta name="meow-links" content="TOKEN"&gt;`（TOKEN 为下方提供的验证 token）。</div>
            </div>
            <form class="mt-4 space-y-3" @submit.prevent="submitApply">
              <input
                v-model.trim="form.site_name"
                type="text"
                required
                maxlength="32"
                placeholder="站点名称 *"
                class="meow-input"
                :class="isNight ? 'meow-input-night' : ''"
              />
              <input
                v-model.trim="form.site_url"
                type="text"
                required
                maxlength="255"
                placeholder="站点地址 * (https://...)"
                class="meow-input"
                :class="isNight ? 'meow-input-night' : ''"
              />
              <input
                v-model.trim="form.avatar_url"
                type="text"
                maxlength="255"
                placeholder="头像地址"
                class="meow-input"
                :class="isNight ? 'meow-input-night' : ''"
              />
              <input
                v-model.trim="form.email"
                type="email"
                required
                maxlength="128"
                placeholder="联系邮箱 *"
                class="meow-input"
                :class="isNight ? 'meow-input-night' : ''"
              />
              <textarea
                v-model.trim="form.description"
                rows="3"
                maxlength="280"
                placeholder="站点简介"
                class="meow-input resize-none"
                :class="isNight ? 'meow-input-night' : ''"
              ></textarea>
              <textarea
                v-model.trim="form.note"
                rows="2"
                maxlength="280"
                placeholder="备注"
                class="meow-input resize-none"
                :class="isNight ? 'meow-input-night' : ''"
              ></textarea>
              <div
                v-if="applyConfig.captcha_enabled"
                class="rounded-2xl border p-3"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg/40' : 'border-meow-line bg-white/50'"
              >
                <div :id="APPLY_CAPTCHA_CONTAINER_ID"></div>
                <p class="mt-2 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                  已启用人机验证，请完成后提交。
                </p>
                <p
                  v-if="!applyConfig.captcha_secret_set"
                  class="mt-1 text-[11px] text-[#e45883]"
                >
                  管理端尚未配置验证 Secret，验证码可能无法通过校验。
                </p>
              </div>
              <button
                type="submit"
                class="meow-btn-primary motion-press w-full"
                :class="isNight ? 'bg-meow-night-accent text-meow-night-bg' : ''"
                :disabled="submitLoading"
              >
                {{ submitLoading ? "提交中..." : "提交申请" }}
              </button>
            </form>
            <div
              v-if="verifyHint.application_id"
              class="mt-3 rounded-2xl border p-3"
              :class="isNight ? 'border-meow-night-line bg-meow-night-bg/40' : 'border-meow-line bg-white/60'"
            >
              <p class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                当前申请编号：#{{ verifyHint.application_id }}（状态：{{ verifyHint.verify_status || "verify_pending" }}）
              </p>
              <div v-if="verifyHint.verify_token" class="mt-2 text-xs">
                <div :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">验证 token：</div>
                <div class="mt-1 flex flex-wrap items-center gap-2">
                  <code class="rounded-lg border px-2 py-1 text-[11px]" :class="isNight ? 'border-meow-night-line bg-meow-night-bg' : 'border-meow-line bg-white'">
                    {{ verifyHint.verify_token }}
                  </code>
                  <button
                    type="button"
                    class="meow-btn-ghost"
                    :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                    @click="copyText(verifyHint.verify_token)"
                  >
                    复制 token
                  </button>
                </div>
                <div class="mt-2 space-y-1" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                  <div>HTTP 文件：在 `/.well-known/meow-links.txt` 写入 token（任意位置包含即可）</div>
                  <div>DNS TXT：添加 `_meow-links` 记录，值包含 token</div>
                  <div class="flex flex-wrap items-center gap-2">
                    <span>Meta：</span>
                    <code class="rounded-lg border px-2 py-1 text-[11px]" :class="isNight ? 'border-meow-night-line bg-meow-night-bg' : 'border-meow-line bg-white'">
                      {{ "<meta name=\"meow-links\" content=\"" + verifyHint.verify_token + "\">" }}
                    </code>
                  </div>
                </div>
              </div>
              <p v-if="verifyHint.verify_deadline" class="mt-2 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                验证有效期至：{{ new Date(verifyHint.verify_deadline * 1000).toLocaleString("zh-CN") }}
              </p>
              <div class="mt-2 flex flex-wrap gap-2">
                <div
                  v-if="applyConfig.captcha_enabled"
                  class="w-full rounded-2xl border p-3"
                  :class="isNight ? 'border-meow-night-line bg-meow-night-bg/40' : 'border-meow-line bg-white/50'"
                >
                  <div :id="EMAIL_CAPTCHA_CONTAINER_ID"></div>
                  <p class="mt-2 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                    邮箱验证需单独完成人机验证。
                  </p>
                </div>
                <button
                  type="button"
                  class="meow-btn-ghost"
                  :disabled="verifyHint.actionLoading"
                  :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                  @click="sendVerifyEmail"
                >
                  发送邮箱验证链接（推荐）
                </button>
                <button
                  type="button"
                  class="meow-btn-ghost"
                  :disabled="verifyHint.actionLoading"
                  :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                  @click="triggerHttpVerify"
                >
                  {{ verifyHint.actionLoading ? "处理中..." : "我已完成 HTTP 验证" }}
                </button>
              </div>
            </div>
            <p v-if="submitError" class="mt-3 text-xs text-[#e45883]">{{ submitError }}</p>
            <p v-if="submitSuccess" class="mt-3 text-xs" :class="isNight ? 'text-meow-night-accent' : 'text-[#2f8f72]'">
              {{ submitSuccess }}
            </p>
          </article>

          <article
            class="meow-card motion-card rounded-3xl p-5"
            :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
          >
            <div class="flex items-center justify-between gap-2">
              <h2 class="font-display text-2xl">本站信息</h2>
              <button
                type="button"
                class="meow-pill motion-press"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : ''"
                @click="copyText(mySiteOneClickText)"
              >
                一键复制
              </button>
            </div>
            <p class="mt-2 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              可以直接复制到你的友链配置中。
            </p>
            <div class="mt-3 space-y-2">
              <div class="site-row">
                <div class="site-key">站点名称</div>
                <div class="site-value">{{ mySiteInfo.site_name }}</div>
                <button type="button" class="site-copy" @click="copyText(mySiteInfo.site_name)">复制</button>
              </div>
              <div class="site-row">
                <div class="site-key">站点简介</div>
                <div class="site-value">{{ mySiteInfo.description }}</div>
                <button type="button" class="site-copy" @click="copyText(mySiteInfo.description)">复制</button>
              </div>
              <div class="site-row">
                <div class="site-key">站点地址</div>
                <div class="site-value">{{ mySiteInfo.site_url }}</div>
                <button type="button" class="site-copy" @click="copyText(mySiteInfo.site_url)">复制</button>
              </div>
              <div class="site-row">
                <div class="site-key">头像地址</div>
                <div class="site-value">{{ mySiteInfo.avatar_url }}</div>
                <button type="button" class="site-copy" @click="copyText(mySiteInfo.avatar_url)">复制</button>
              </div>
            </div>
            <p v-if="copyStatus" class="mt-2 text-xs" :class="isNight ? 'text-meow-night-accent' : 'text-[#2f8f72]'">
              {{ copyStatus }}
            </p>
          </article>
        </div>
      </section>
      <footer class="mt-16 text-center text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
        © 2026 Meowhuan ·
        <a
          class="underline underline-offset-4"
          href="https://beian.miit.gov.cn/"
          target="_blank"
          rel="noreferrer"
        >
          鲁ICP备2026020825号
        </a>
      </footer>
    </main>
  </div>
  <div
    v-if="modal.open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4"
    @click.self="closeModal"
  >
    <div
      class="w-full max-w-sm rounded-2xl border p-4 shadow-lg"
      :class="isNight ? 'border-meow-night-line bg-meow-night-card text-meow-night-ink' : 'border-meow-line bg-white text-meow-ink'"
    >
      <div class="text-sm font-700">{{ modal.title }}</div>
      <p class="mt-2 text-xs leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
        {{ modal.message }}
      </p>
      <div class="mt-3 flex justify-end">
        <button
          type="button"
          class="meow-btn-ghost"
          :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-bg' : ''"
          @click="closeModal"
        >
          知道了
        </button>
      </div>
    </div>
  </div>
</template>

<style>
.page-fade {
  animation: pageFade 0.72s cubic-bezier(0.22, 1.2, 0.36, 1) both;
}

@keyframes pageFade {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

.meow-bg {
  position: relative;
}

.meow-bg::before {
  content: "";
  position: fixed;
  inset: 0;
  pointer-events: none;
  opacity: 0.05;
  background-image: url("data:image/svg+xml;utf8,%3Csvg width='120' height='120' viewBox='0 0 120 120' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%23bca6d9'%3E%3Ccircle cx='24' cy='24' r='6'/%3E%3Ccircle cx='44' cy='18' r='6'/%3E%3Ccircle cx='64' cy='24' r='6'/%3E%3Ccircle cx='32' cy='48' r='12'/%3E%3Ccircle cx='84' cy='84' r='6'/%3E%3Ccircle cx='104' cy='78' r='6'/%3E%3Ccircle cx='96' cy='56' r='6'/%3E%3Ccircle cx='88' cy='100' r='12'/%3E%3C/g%3E%3C/svg%3E");
  background-size: 140px 140px;
}

.meow-bg.meow-night::before {
  opacity: 0.04;
  background-image: url("data:image/svg+xml;utf8,%3Csvg width='120' height='120' viewBox='0 0 120 120' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%235a6bb8'%3E%3Ccircle cx='24' cy='24' r='6'/%3E%3Ccircle cx='44' cy='18' r='6'/%3E%3Ccircle cx='64' cy='24' r='6'/%3E%3Ccircle cx='32' cy='48' r='12'/%3E%3Ccircle cx='84' cy='84' r='6'/%3E%3Ccircle cx='104' cy='78' r='6'/%3E%3Ccircle cx='96' cy='56' r='6'/%3E%3Ccircle cx='88' cy='100' r='12'/%3E%3C/g%3E%3C/svg%3E");
}

.cord-switch {
  position: fixed;
  top: 4px;
  right: 280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 0;
  cursor: pointer;
  padding: 0;
  z-index: 20;
}

.cord-line {
  width: 2px;
  height: 36px;
  background: #e9d9ea;
  border-radius: 999px;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.5);
}

.cord-knob {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 999px;
  background: #fff7fb;
  color: #2b1d2a;
  border: 1px solid #e9d9ea;
  box-shadow: 0 8px 18px rgba(47, 20, 47, 0.12);
}

.cord-label {
  height: 8px;
}

.cord-switch-night .cord-line {
  background: #332b55;
}

.cord-switch-night .cord-knob {
  background: #241f3d;
  color: #f3e9ff;
  border-color: #332b55;
}

.motion-card {
  transition: transform 0.45s ease, box-shadow 0.45s ease;
}

.motion-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 16px 32px rgba(47, 20, 47, 0.14);
}

.motion-press {
  transition: transform 0.2s ease;
}

.motion-press:hover {
  transform: translateY(-2px);
}

.friend-card {
  display: flex;
  gap: 12px;
  border: 1px solid rgba(233, 217, 234, 0.95);
  border-radius: 16px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.72);
  text-decoration: none;
  color: inherit;
}

.friend-card-night {
  border-color: rgba(74, 64, 110, 0.9);
  background: rgba(35, 28, 58, 0.85);
}

.meow-input {
  width: 100%;
  border: 1px solid rgba(233, 217, 234, 0.95);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.82);
  color: #2b1d2a;
  padding: 10px 12px;
  font-size: 13px;
  outline: none;
}

.meow-input:focus {
  box-shadow: 0 0 0 3px rgba(255, 122, 182, 0.2);
}

.meow-input-night {
  border-color: rgba(74, 64, 110, 0.9);
  background: rgba(26, 23, 45, 0.85);
  color: #f3e9ff;
}

.meow-input-night:focus {
  box-shadow: 0 0 0 3px rgba(136, 243, 255, 0.18);
}

.site-row {
  display: grid;
  grid-template-columns: 68px 1fr auto;
  gap: 8px;
  align-items: center;
  border: 1px solid rgba(233, 217, 234, 0.95);
  border-radius: 12px;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.72);
}

.meow-night .site-row {
  border-color: rgba(74, 64, 110, 0.9);
  background: rgba(26, 23, 45, 0.82);
}

.site-key {
  font-size: 11px;
  color: #7e6a86;
}

.meow-night .site-key {
  color: #b8a6d8;
}

.site-value {
  font-size: 12px;
  word-break: break-all;
}

.site-copy {
  border: 1px solid rgba(233, 217, 234, 0.95);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.85);
  color: #2b1d2a;
  font-size: 11px;
  padding: 3px 10px;
  cursor: pointer;
}

.meow-night .site-copy {
  border-color: rgba(74, 64, 110, 0.9);
  background: rgba(35, 28, 58, 0.88);
  color: #f3e9ff;
}

@media (max-width: 640px) {
  .cord-switch {
    right: 100px;
  }

  .site-row {
    grid-template-columns: 1fr;
  }
}
</style>
