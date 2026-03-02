import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

const STATUS_URL = "https://m.ratf.cn/status";
const SCHEDULE_URL = "https://m.ratf.cn/schedule";
const BLOG_URL = "https://m.ratf.cn/blog";
const VISITOR_URL = "https://m.ratf.cn/visitor";
const VISITOR_VISIT_URL = "https://m.ratf.cn/visitor/visit";
const VISITOR_ID_KEY = "meow-visitor-id";
const QUOTE_FALLBACK = "今天也要温柔一点。";
const HRT_TARGET_DATE = "2026-01-16T00:00:00+08:00";
const SITE_STARTED_AT = "2026-02-06T19:56:27+08:00";

function createThemeUrl(isNight) {
  const file = isNight ? "/giscus-dark.css" : "/giscus.css";
  const url = new URL(file, window.location.origin);
  url.searchParams.set("v", Date.now().toString());
  return url.toString();
}

export function useHomepageState() {
  const time = ref("");
  const date = ref("");
  const calendar = ref({ day: "", month: "", weekday: "" });
  const hrtCountdownText = ref("");
  const hrtDateLabel = "2026.01.16";
  const siteUptimeText = ref("");
  const isNight = ref(false);
  const showIntro = ref(true);

  const quoteText = ref("加载中…");
  const quoteFrom = ref("");
  const quoteError = ref(false);
  const quoteLoading = ref(false);
  const nextQuoteAt = ref(0);

  const statusList = ref([]);
  const statusError = ref(false);
  const statusLoading = ref(false);
  const statusUpdatedAt = ref(0);
  const statusNextAt = ref(0);

  const scheduleList = ref([]);
  const scheduleError = ref(false);
  const scheduleLoading = ref(false);
  const scheduleUpdatedAt = ref(0);
  const scheduleNextAt = ref(0);

  const blogList = ref([]);
  const blogError = ref(false);
  const blogLoading = ref(false);
  const blogUpdatedAt = ref(0);
  const blogNextAt = ref(0);

  const visitorCount = ref(0);
  const visitorToday = ref(0);
  const visitorMonth = ref(0);
  const visitorLoading = ref(false);
  const visitorError = ref(false);
  const visitorUpdatedAt = ref(0);
  const visitorNextAt = ref(0);

  const themeMedia = ref(null);

  const quoteCooldownMs = 1500;
  const statusCooldownMs = 5000;
  const scheduleCooldownMs = 5000;
  const blogCooldownMs = 5000;
  const visitorCooldownMs = 5000;

  const timers = {
    clock: 0,
    intro: 0,
    statusPoll: 0,
    schedulePoll: 0,
    blogPoll: 0
  };

  const hasOnlineDevice = computed(() =>
    statusList.value.some((item) => item?.online)
  );
  const allDevicesOffline = computed(
    () => statusList.value.length > 0 && statusList.value.every((item) => !item?.online)
  );
  const statusSummaryText = computed(() => {
    if (statusLoading.value && statusList.value.length === 0) return "加载中";
    if (hasOnlineDevice.value) return "营业中";
    if (allDevicesOffline.value) return "在忙/睡觉";
    return "暂时无法获取";
  });
  const statusSummaryClass = computed(() => {
    if (hasOnlineDevice.value) {
      return isNight.value ? "text-meow-night-accent" : "text-meow-accent";
    }
    return isNight.value ? "text-meow-night-soft" : "text-meow-soft";
  });

  const splitTags = (tagValue) => {
    if (!tagValue || typeof tagValue !== "string") return [];
    return tagValue
      .split(/[,，]/)
      .map((item) => item.trim())
      .filter(Boolean);
  };

  const updateClock = () => {
    const now = new Date();
    time.value = now.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit"
    });
    date.value = now.toLocaleDateString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      weekday: "short"
    });
    calendar.value = {
      day: now.toLocaleDateString("zh-CN", { day: "2-digit" }),
      month: now.toLocaleDateString("zh-CN", { month: "2-digit" }),
      weekday: now.toLocaleDateString("zh-CN", { weekday: "short" })
    };

    const hrtTargetDate = new Date(HRT_TARGET_DATE);
    const nowDay = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const targetDay = new Date(
      hrtTargetDate.getFullYear(),
      hrtTargetDate.getMonth(),
      hrtTargetDate.getDate()
    );
    const dayDiff = Math.floor((targetDay.getTime() - nowDay.getTime()) / (24 * 60 * 60 * 1000));
    if (dayDiff > 0) {
      hrtCountdownText.value = `还有 ${dayDiff + 1} 天`;
    } else {
      hrtCountdownText.value = `已进行第 ${Math.abs(dayDiff) + 1} 天`;
    }

    const siteStartedTime = new Date(SITE_STARTED_AT).getTime();
    const uptimeSeconds = Math.max(0, Math.floor((now.getTime() - siteStartedTime) / 1000));
    const days = Math.floor(uptimeSeconds / 86400);
    const hours = Math.floor((uptimeSeconds % 86400) / 3600);
    const minutes = Math.floor((uptimeSeconds % 3600) / 60);
    const seconds = uptimeSeconds % 60;
    siteUptimeText.value = `${days}天 ${String(hours).padStart(2, "0")}时 ${String(minutes).padStart(2, "0")}分 ${String(seconds).padStart(2, "0")}秒`;
  };

  const canFetchQuote = () => Date.now() >= nextQuoteAt.value;
  const canFetchStatus = () => Date.now() >= statusNextAt.value;
  const canFetchSchedule = () => Date.now() >= scheduleNextAt.value;
  const canFetchBlog = () => Date.now() >= blogNextAt.value;
  const canFetchVisitor = () => Date.now() >= visitorNextAt.value;

  const fetchQuote = () => {
    if (!canFetchQuote()) return;
    quoteLoading.value = true;
    nextQuoteAt.value = Date.now() + quoteCooldownMs;
    fetch("https://v1.meowra.cn")
      .then((res) => res.json())
      .then((data) => {
        quoteText.value = data?.hitokoto || QUOTE_FALLBACK;
        quoteFrom.value = data?.from ? `—— ${data.from}` : "";
        quoteError.value = false;
      })
      .catch(() => {
        quoteText.value = QUOTE_FALLBACK;
        quoteFrom.value = "";
        quoteError.value = true;
      })
      .finally(() => {
        quoteLoading.value = false;
      });
  };

  const fetchStatus = async () => {
    if (!canFetchStatus()) return;
    statusNextAt.value = Date.now() + statusCooldownMs;
    statusLoading.value = true;
    try {
      const res = await fetch(STATUS_URL);
      if (!res.ok) throw new Error("status fetch failed");
      const data = await res.json();
      if (Array.isArray(data)) {
        statusList.value = data;
        statusError.value = false;
        statusUpdatedAt.value = Date.now();
      }
    } catch {
      statusError.value = true;
    } finally {
      statusLoading.value = false;
    }
  };

  const fetchSchedule = async () => {
    if (!canFetchSchedule()) return;
    scheduleNextAt.value = Date.now() + scheduleCooldownMs;
    scheduleLoading.value = true;
    try {
      const res = await fetch(SCHEDULE_URL);
      if (!res.ok) throw new Error("schedule fetch failed");
      const data = await res.json();
      if (Array.isArray(data)) {
        scheduleList.value = data;
        scheduleError.value = false;
        scheduleUpdatedAt.value = Date.now();
      }
    } catch {
      scheduleError.value = true;
    } finally {
      scheduleLoading.value = false;
    }
  };

  const fetchBlog = async () => {
    if (!canFetchBlog()) return;
    blogNextAt.value = Date.now() + blogCooldownMs;
    blogLoading.value = true;
    try {
      const res = await fetch(BLOG_URL);
      if (!res.ok) throw new Error("blog fetch failed");
      const data = await res.json();
      if (Array.isArray(data)) {
        blogList.value = data;
        blogError.value = false;
        blogUpdatedAt.value = Date.now();
      }
    } catch {
      blogError.value = true;
    } finally {
      blogLoading.value = false;
    }
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

  const initVisitorId = () => {
    const existing = localStorage.getItem(VISITOR_ID_KEY);
    if (existing) return existing;
    const id = `v-${crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(16).slice(2)}`;
    localStorage.setItem(VISITOR_ID_KEY, id);
    return id;
  };

  const fetchVisitorStats = async () => {
    if (!canFetchVisitor()) return;
    visitorNextAt.value = Date.now() + visitorCooldownMs;
    visitorLoading.value = true;
    try {
      const res = await fetch(VISITOR_URL);
      if (!res.ok) throw new Error("visitor fetch failed");
      const data = await res.json();
      visitorToday.value = Number(data?.today || 0);
      visitorMonth.value = Number(data?.month || 0);
      visitorCount.value = Number(data?.total || 0);
      visitorUpdatedAt.value = Date.now();
      visitorError.value = false;
    } catch {
      visitorError.value = true;
    } finally {
      visitorLoading.value = false;
    }
  };

  const recordVisitorVisit = async () => {
    const visitorId = initVisitorId();
    try {
      await fetch(VISITOR_VISIT_URL, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ visitor_id: visitorId })
      });
    } finally {
      fetchVisitorStats();
    }
  };

  const loadGiscus = () => {
    const container = document.getElementById("giscus");
    if (!container || container.hasChildNodes()) return;
    const script = document.createElement("script");
    script.src = "https://giscus.app/client.js";
    script.async = true;
    script.crossOrigin = "anonymous";
    script.setAttribute("data-repo", "meowhuan/Personal_Homepage");
    script.setAttribute("data-repo-id", "R_kgDORKC6Nw");
    script.setAttribute("data-category", "Announcements");
    script.setAttribute("data-category-id", "DIC_kwDORKC6N84C19nh");
    script.setAttribute("data-mapping", "pathname");
    script.setAttribute("data-strict", "0");
    script.setAttribute("data-reactions-enabled", "0");
    script.setAttribute("data-emit-metadata", "0");
    script.setAttribute("data-input-position", "top");
    script.setAttribute("data-theme", createThemeUrl(isNight.value));
    script.setAttribute("data-lang", "zh-CN");
    container.appendChild(script);
  };

  const updateGiscusTheme = () => {
    const iframe = document.querySelector("iframe.giscus-frame");
    if (!iframe) return;
    iframe.contentWindow?.postMessage(
      { giscus: { setConfig: { theme: createThemeUrl(isNight.value) } } },
      "https://giscus.app"
    );
  };

  watch(isNight, updateGiscusTheme);

  onMounted(() => {
    updateClock();
    timers.clock = window.setInterval(updateClock, 1000);

    fetchQuote();
    fetchStatus();
    fetchSchedule();
    fetchBlog();
    initVisitorId();
    fetchVisitorStats();
    timers.statusPoll = window.setInterval(fetchStatus, 60000);
    timers.schedulePoll = window.setInterval(fetchSchedule, 120000);
    timers.blogPoll = window.setInterval(fetchBlog, 180000);

    const savedTheme = localStorage.getItem("meow-theme");
    if (savedTheme) {
      isNight.value = savedTheme === "night";
    } else if (window.matchMedia) {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      isNight.value = media.matches;
      themeMedia.value = media;
      media.addEventListener("change", onSystemThemeChange);
    }

    loadGiscus();
    recordVisitorVisit();
    timers.intro = window.setTimeout(() => {
      showIntro.value = false;
    }, 620);
  });

  onBeforeUnmount(() => {
    if (timers.clock) window.clearInterval(timers.clock);
    if (timers.intro) window.clearTimeout(timers.intro);
    if (timers.statusPoll) window.clearInterval(timers.statusPoll);
    if (timers.schedulePoll) window.clearInterval(timers.schedulePoll);
    if (timers.blogPoll) window.clearInterval(timers.blogPoll);
    if (themeMedia.value) {
      themeMedia.value.removeEventListener("change", onSystemThemeChange);
    }
  });

  return {
    time,
    date,
    calendar,
    hrtCountdownText,
    hrtDateLabel,
    siteUptimeText,
    isNight,
    showIntro,
    quoteText,
    quoteFrom,
    quoteError,
    quoteLoading,
    statusList,
    statusError,
    statusLoading,
    statusUpdatedAt,
    scheduleList,
    scheduleError,
    scheduleLoading,
    scheduleUpdatedAt,
    blogList,
    blogError,
    blogLoading,
    blogUpdatedAt,
    visitorCount,
    visitorToday,
    visitorMonth,
    visitorLoading,
    visitorError,
    visitorUpdatedAt,
    hasOnlineDevice,
    allDevicesOffline,
    statusSummaryText,
    statusSummaryClass,
    splitTags,
    toggleTheme,
    canFetchQuote,
    fetchQuote,
    canFetchStatus,
    fetchStatus,
    canFetchSchedule,
    fetchSchedule,
    canFetchBlog,
    fetchBlog
  };
}
