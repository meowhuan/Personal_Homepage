<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

const time = ref("");
const date = ref("");
const calendar = ref({ day: "", month: "", weekday: "" });
let timer = 0;
const isNight = ref(false);
const quoteText = ref("加载中…");
const quoteFrom = ref("");
const quoteError = ref(false);
const quoteLoading = ref(false);
const nextQuoteAt = ref(0);
const cooldownMs = 1500;
const statusList = ref([]);
const statusError = ref(false);
const statusUrl = "https://m.ratf.cn/status";
const statusLoading = ref(false);
const statusUpdatedAt = ref(0);
const statusNextAt = ref(0);
const statusCooldownMs = 5000;
const scheduleList = ref([]);
const scheduleError = ref(false);
const scheduleUrl = "https://m.ratf.cn/schedule";
const scheduleLoading = ref(false);
const scheduleUpdatedAt = ref(0);
const scheduleNextAt = ref(0);
const scheduleCooldownMs = 5000;
const visitorCount = ref(0);
const visitorToday = ref(0);
const visitorMonth = ref(0);
const visitorLoading = ref(false);
const visitorError = ref(false);
const visitorUpdatedAt = ref(0);
const visitorNextAt = ref(0);
const visitorCooldownMs = 5000;
const visitorUrl = "https://m.ratf.cn/visitor";
const visitorVisitUrl = "https://m.ratf.cn/visitor/visit";
const visitorIdKey = "meow-visitor-id";
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
};

onMounted(() => {
  updateClock();
  timer = window.setInterval(updateClock, 1000);
  fetchQuote();
  fetchStatus();
  fetchSchedule();
  initVisitorId();
  fetchVisitorStats();
  setInterval(fetchStatus, 60000);
  setInterval(fetchSchedule, 120000);
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) {
    isNight.value = savedTheme === "night";
  } else if (window.matchMedia) {
    isNight.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  loadGiscus();
  recordVisitorVisit();
});

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer);
});

const canFetchQuote = () => Date.now() >= nextQuoteAt.value;

const fetchQuote = () => {
  if (!canFetchQuote()) return;
  quoteLoading.value = true;
  nextQuoteAt.value = Date.now() + cooldownMs;
  fetch("https://v1.meowra.cn")
    .then((res) => res.json())
    .then((data) => {
      quoteText.value = data?.hitokoto || "今天也要温柔一点。";
      quoteFrom.value = data?.from ? `—— ${data.from}` : "";
      quoteError.value = false;
    })
    .catch(() => {
      quoteText.value = "今天也要温柔一点。";
      quoteFrom.value = "";
      quoteError.value = true;
    })
    .finally(() => {
      quoteLoading.value = false;
    });
};

const canFetchStatus = () => Date.now() >= statusNextAt.value;
const canFetchSchedule = () => Date.now() >= scheduleNextAt.value;

const fetchStatus = async () => {
  if (!canFetchStatus()) return;
  statusNextAt.value = Date.now() + statusCooldownMs;
  statusLoading.value = true;
  try {
    const res = await fetch(statusUrl);
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
    const res = await fetch(scheduleUrl);
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

const toggleTheme = () => {
  isNight.value = !isNight.value;
  localStorage.setItem("meow-theme", isNight.value ? "night" : "day");
};

const initVisitorId = () => {
  const existing = localStorage.getItem(visitorIdKey);
  if (existing) return existing;
  const id = `v-${crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(16).slice(2)}`;
  localStorage.setItem(visitorIdKey, id);
  return id;
};

const canFetchVisitor = () => Date.now() >= visitorNextAt.value;

const fetchVisitorStats = async () => {
  if (!canFetchVisitor()) return;
  visitorNextAt.value = Date.now() + visitorCooldownMs;
  visitorLoading.value = true;
  try {
    const res = await fetch(visitorUrl);
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
    await fetch(visitorVisitUrl, {
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
  script.setAttribute(
    "data-theme",
    isNight.value
      ? `https://raw.githack.com/meowhuan/Personal_Homepage/main/public/giscus-dark.css?v=${Date.now()}`
      : `https://raw.githack.com/meowhuan/Personal_Homepage/main/public/giscus.css?v=${Date.now()}`
  );
  script.setAttribute("data-lang", "zh-CN");
  container.appendChild(script);
};

const updateGiscusTheme = () => {
  const iframe = document.querySelector("iframe.giscus-frame");
  if (!iframe) return;
  iframe.contentWindow?.postMessage(
    {
      giscus: {
        setConfig: {
          theme: isNight.value
          ? `https://raw.githack.com/meowhuan/Personal_Homepage/main/public/giscus-dark.css?v=${Date.now()}`
          : `https://raw.githack.com/meowhuan/Personal_Homepage/main/public/giscus.css?v=${Date.now()}`
        }
      }
    },
    "https://giscus.app"
  );
};

watch(isNight, () => {
  updateGiscusTheme();
});
</script>

<template>
  <div
    class="min-h-screen font-body page-fade transition-colors duration-700 ease-in-out meow-bg"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink meow-night'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink meow-day'"
  >
    <div class="relative overflow-hidden">
      <div
        class="pointer-events-none absolute -left-32 -top-24 h-80 w-80 rounded-[45%_55%_60%_40%/50%_60%_40%_50%] blur-3xl opacity-70 animate-floaty"
        :class="isNight
          ? 'bg-[radial-gradient(circle_at_top,_#3a2b6f,_transparent_65%)]'
          : 'bg-[radial-gradient(circle_at_top,_#ffd4e6,_transparent_65%)]'"
      ></div>
      <div
        class="pointer-events-none absolute -right-32 top-24 h-96 w-96 rounded-[55%_45%_45%_55%/45%_55%_45%_55%] blur-3xl opacity-70 animate-floaty"
        :class="isNight
          ? 'bg-[radial-gradient(circle_at_top,_#1d5c7a,_transparent_65%)]'
          : 'bg-[radial-gradient(circle_at_top,_#c8f6ed,_transparent_65%)]'"
      ></div>

      <div class="mx-auto w-[min(1100px,92vw)] pb-20 pt-8 relative">
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

        <nav class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-3">
            <img
              src="/logo.png"
              alt="Meowhuan logo"
              class="h-10 w-10 rounded-full border bg-white/70 object-cover shadow-sm"
              :class="isNight ? 'border-meow-night-line' : 'border-meow-line'"
            />
            <div class="font-display text-xl tracking-wide">Meowhuan</div>
          </div>
          <div class="hidden items-center gap-5 text-sm md:flex" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
            <a class="nav-link" :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#about">关于我</a>
            <a class="nav-link" :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#schedule">行程表</a>
            <a class="nav-link" :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#stuff">我在做</a>
            <a class="nav-link" :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="/blog.html">博客</a>
            <a class="nav-link" :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#contact">联系</a>
          </div>
        </nav>

        <div class="mt-6 grid gap-4 md:grid-cols-[1fr_auto] md:items-stretch">
          <div
            class="meow-card motion-card flex h-full min-h-[96px] items-start gap-3 rounded-3xl px-5 py-4 text-xs backdrop-blur"
            style="--float-delay: 0.2s"
            :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
          >
            <span class="text-base">💬</span>
            <div class="flex-1 leading-tight">
              <div class="text-[11px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">今日一句</div>
              <div class="mt-1 text-sm font-600" :class="isNight ? 'text-meow-night-ink' : 'text-meow-ink'">{{ quoteText }}</div>
              <div class="mt-1 text-[11px]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                {{ quoteFrom || (quoteError ? "暂时无法获取" : " ") }}
              </div>
            </div>
            <button
              class="meow-pill motion-press px-2 py-1 text-[11px] transition-opacity"
              type="button"
              :disabled="!canFetchQuote() || quoteLoading"
              :class="[
                (!canFetchQuote() || quoteLoading) ? 'opacity-50' : '',
                isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''
              ]"
              @click="fetchQuote()"
            >
              {{ quoteLoading ? "加载中" : "换一句" }}
            </button>
          </div>

          <div
            class="meow-card motion-card flex h-full min-h-[96px] items-center gap-4 rounded-3xl px-5 py-4 text-xs backdrop-blur md:justify-self-end"
            style="--float-delay: 0.6s"
            :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
          >
            <div
              class="flex h-16 w-16 flex-col items-center justify-center rounded-2xl shadow-[0_8px_18px_rgba(47,20,47,0.12)]"
              :class="isNight ? 'bg-meow-night-bg text-meow-night-ink' : 'bg-white/70 text-meow-ink'"
            >
              <div class="text-[11px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">月</div>
              <div class="text-xl font-700">{{ calendar.month }}</div>
              <div class="text-[11px]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">{{ calendar.weekday }}</div>
            </div>
            <div class="leading-tight">
              <div class="text-[11px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">本地时间</div>
              <div class="text-lg font-700" :class="isNight ? 'text-meow-night-ink' : 'text-meow-ink'">{{ time }}</div>
              <div class="text-[11px]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">日期 {{ calendar.month }}/{{ calendar.day }}</div>
            </div>
          </div>
        </div>

        <section class="mt-12 grid gap-8 md:grid-cols-[1.15fr_0.85fr]">
          <div class="space-y-6">
            <span class="meow-pill">🐾 Meow 个人档案</span>
            <h1 class="font-display text-4xl leading-tight sm:text-5xl">
              处于互联网边缘区的小猫。
            </h1>
            <p class="text-base leading-relaxed text-meow-soft">
              09 年小朋友，在读，来自广东湛江。喜欢睡觉、玩明日方舟、喝奶茶（怎么都吃不胖）。
              爱好计算机，随缘做任何事。HRT：2026.1.16。想做女孩子。
              中度抑郁（状态未知）。讨厌不喜欢的人。不要摸我头！脸也不行！！
            </p>
            <div class="flex flex-wrap gap-3">
              <a
                class="meow-btn-primary motion-press"
                :class="isNight ? 'bg-meow-night-accent text-meow-night-bg' : ''"
                href="mailto:meowhuan@qq.com"
              >
                Email me
              </a>
              <a
                class="meow-btn-ghost motion-press"
                :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                href="https://list.meowra.cn"
                target="_blank"
                rel="noreferrer"
              >
                OpenList / 下载站
              </a>
              <a
                class="meow-btn-ghost motion-press"
                :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                href="/donate.html"
              >
                喵喵补给站
              </a>
              <a
                class="meow-btn-ghost motion-press"
                :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                href="/blog.html"
              >
                喵喵博客
              </a>
            </div>

          </div>

          <div
            class="meow-card motion-card quick-card p-6"
            style="--float-delay: 0.4s"
            :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
          >
            <h2 class="font-display text-2xl">快速了解我</h2>
            <div class="quick-card-body">
            <div class="mt-4 space-y-2 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              <p>📌 城市：广东湛江</p>
              <p>🎒 方向：随缘</p>
              <p>☕ 标签：MTF / HRT / 药娘 / 无证含糖</p>
            </div>
            <div class="mt-5 flex flex-wrap gap-2">
              <span class="meow-pill">#日常记录</span>
              <span class="meow-pill">#开源项目</span>
              <span class="meow-pill">#情绪垃圾桶</span>
            </div>

            <div class="mt-4 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              <div class="flex items-center justify-between gap-2">
                <div class="text-[11px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                  在线状态
                </div>
                <button
                  class="meow-pill motion-press px-2 py-0.5 text-[11px]"
                  type="button"
                  :class="[
                    (!canFetchStatus() || statusLoading) ? 'opacity-50' : '',
                    isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''
                  ]"
                  :disabled="statusLoading || !canFetchStatus()"
                  @click="fetchStatus()"
                >
                  {{ statusLoading ? "刷新中" : (canFetchStatus() ? "刷新" : "冷却中") }}
                </button>
              </div>
              <div class="mt-2 flex items-center justify-between gap-2 text-[11px]">
                <span :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">设备状态</span>
                <span :class="statusSummaryClass">{{ statusSummaryText }}</span>
              </div>
              <div class="mt-1 text-[11px]" v-if="statusUpdatedAt">
                更新于 {{ new Date(statusUpdatedAt).toLocaleTimeString("zh-CN") }}
              </div>
              <div v-if="statusError" class="mt-2">暂时无法获取</div>
              <div v-else class="mt-2 space-y-1 status-list">
                <div
                  v-for="item in statusList.slice(0, 3)"
                  :key="item.device_id"
                  class="flex items-center justify-between gap-2"
                >
                  <span class="truncate">{{ item.device_name }}</span>
                  <span
                    class="text-[11px]"
                    :class="item.online ? (isNight ? 'text-meow-night-accent' : 'text-meow-accent') : (isNight ? 'text-meow-night-soft' : 'text-meow-soft')"
                  >
                    {{ item.online ? "在线" : "离线" }}
                  </span>
                </div>
              </div>
            </div>
            </div>
          </div>
        </section>

        <section id="about" class="mt-16">
          <h2 class="font-display text-2xl">关于我</h2>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.1s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">我在意的事</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                并没有在意的事 ( •̥́ ˍ •̀ू )，只想过好每一天，和喜欢的人一起做喜欢的事。
              </p>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.35s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">我的兴趣</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                编程、记录、睡觉、奶茶、明日方舟，还有可爱的设计和配色。
              </p>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.6s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">我在寻找</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                貌似并没有什么想寻找的东西，但如果有的话，希望是能让我开心的事物和人吧。
              </p>
            </article>
          </div>
        </section>

        <section id="stuff" class="mt-16">
          <h2 class="font-display text-2xl">我最近在做</h2>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <div
              id="schedule"
              class="meow-window meow-card motion-card p-5 md:col-span-3"
              style="--float-delay: 0.15s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
            >
              <div class="meow-window-bar">
                <span class="meow-window-dots"></span>
                <span class="meow-window-title">Meow Schedule</span>
                <button
                  class="meow-pill motion-press px-2 py-0.5 text-[11px]"
                  type="button"
                  :class="[
                    (!canFetchSchedule() || scheduleLoading) ? 'opacity-50' : '',
                    isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''
                  ]"
                  :disabled="scheduleLoading || !canFetchSchedule()"
                  @click="fetchSchedule()"
                >
                  {{ scheduleLoading ? "刷新中" : (canFetchSchedule() ? "刷新" : "冷却中") }}
                </button>
              </div>
              <div class="meow-window-body">
                <div class="text-[11px]" v-if="scheduleUpdatedAt">
                  更新于 {{ new Date(scheduleUpdatedAt).toLocaleTimeString("zh-CN") }}
                </div>
                <div v-if="scheduleError" class="mt-2 text-sm">暂时无法获取</div>
                <div v-else-if="scheduleList.length === 0" class="mt-2 text-sm">暂无行程</div>
                <div v-else class="mt-3 space-y-3">
                  <div
                    v-for="item in scheduleList"
                    :key="item.id"
                    class="meow-window-item"
                  >
                    <div class="meow-window-time">{{ item.time }}</div>
                    <div class="meow-window-content">
                      <div class="meow-window-titleline">
                        <span class="meow-window-name">{{ item.title }}</span>
                        <span v-if="item.tag" class="meow-pill">{{ item.tag }}</span>
                      </div>
                      <div v-if="item.location" class="meow-window-meta">地点：{{ item.location }}</div>
                      <div v-if="item.note" class="meow-window-note">{{ item.note }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.1s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">Oyama's HRT Tracker</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                为记录与跟踪 HRT 过程的前端小工具编写的后端程序，基本完善，不定期维护。
              </p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                href="https://github.com/meowhuan/Oyama-s-HRT-Tracker"
                target="_blank"
                rel="noreferrer"
              >
                查看项目
              </a>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.35s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">技能积累</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                随缘学习新技能，但是不知道学什么好，感觉什么都想学又什么都学不好。
              </p>
              <span class="meow-pill motion-press mt-4">学习中</span>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.6s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">小记录</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                偶尔记录一些日常小事，想记录但又不想记录，感觉有点矛盾 ( •̥́ ˍ •̀ू )。
              </p>
              <span class="meow-pill motion-press mt-4">随缘更新</span>
            </article>
          </div>
        </section>

        <section id="blog" class="mt-16">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="font-display text-2xl">博客更新</h2>
            <a
              class="meow-pill motion-press"
              :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
              href="/blog.html"
            >
              查看全部
            </a>
          </div>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.1s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <span class="meow-pill">日常</span>
              <h3 class="mt-3 text-base font-600">博客开张：给自己留一个角落</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                从这里开始记录，保持轻量更新，慢慢把每个阶段留住。
              </p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                href="/blog.html?post=welcome-to-blog"
              >
                阅读
              </a>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.35s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <span class="meow-pill">开发</span>
              <h3 class="mt-3 text-base font-600">主页更新日志：视觉和结构的几次调整</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                记录设计思路和结构变化，方便后续继续迭代。
              </p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                href="/blog.html?post=homepage-notes"
              >
                阅读
              </a>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.6s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <span class="meow-pill">碎碎念</span>
              <h3 class="mt-3 text-base font-600">一些小事：奶茶、代码和慢慢来</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                生活和开发都在缓慢推进，先把步子迈稳。
              </p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                href="/blog.html?post=small-things"
              >
                阅读
              </a>
            </article>
          </div>
        </section>

        <section id="contact" class="mt-16">
          <h2 class="font-display text-2xl">联系我</h2>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.1s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">邮箱</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">meowhuan@qq.com</p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
                href="mailto:meowhuan@qq.com"
              >
                发送邮件
              </a>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.35s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">GitHub</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">github.com/meowhuan</p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
                href="https://github.com/meowhuan"
                target="_blank"
                rel="noreferrer"
              >
                打开主页
              </a>
            </article>
            <article
              class="meow-card motion-card p-5"
              style="--float-delay: 0.6s"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <h3 class="text-base font-600">X</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">x.com/meow_huan</p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
                href="https://x.com/meow_huan"
                target="_blank"
                rel="noreferrer"
              >
                打开主页
              </a>
            </article>
          </div>
        </section>

        <section id="visitor" class="mt-16">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="font-display text-2xl" :class="isNight ? 'text-meow-night-ink' : ''">喵喵计数器</h2>
            <span class="meow-pill meow-pill-strong">自动记录</span>
          </div>
          <div
            class="meow-card motion-card mt-4 grid gap-3 rounded-[20px] px-4 py-3 md:grid-cols-[1fr_auto]"
            style="--float-delay: 0.2s"
            :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
          >
            <div class="flex flex-col justify-between gap-2">
              <div>
                <div class="meow-title text-[13px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-ink' : 'text-meow-ink'">喵爪印记</div>
                <p class="mt-2 text-[12px] leading-[1.7]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                  每台设备每天留一次爪印。
                </p>
              </div>
              <div class="text-[11px] flex items-center gap-2" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                <span v-if="visitorLoading">统计更新中</span>
                <span v-else-if="visitorError" class="flex items-center gap-2">
                  <span class="status-dot status-dot-error"></span>
                  <span>统计获取失败</span>
                  <svg class="meow-sad" viewBox="0 0 64 64" aria-hidden="true">
                    <circle cx="30" cy="32" r="14" fill="none" stroke="currentColor" stroke-width="3" />
                    <path d="M22 34c4 2 8 2 12 0" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                    <path d="M24 28c2-3 6-4 10-2" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                    <path d="M12 40c8 2 12 6 14 12" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                  </svg>
                </span>
                <span v-else-if="visitorUpdatedAt">更新于 {{ new Date(visitorUpdatedAt).toLocaleTimeString("zh-CN") }}</span>
              </div>
            </div>
            <div class="grid gap-2 sm:grid-cols-3">
              <div
                class="paw-card rounded-[20px] border px-3 py-2"
                :class="isNight
                  ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink'
                  : 'border-meow-line bg-white/70 text-meow-ink'"
              >
                <div class="text-[10px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">今日</div>
                <div class="mt-1 text-3xl font-700">{{ visitorToday }}</div>
              </div>
              <div
                class="paw-card rounded-[20px] border px-3 py-2"
                :class="isNight
                  ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink'
                  : 'border-meow-line bg-white/70 text-meow-ink'"
              >
                <div class="text-[10px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">本月</div>
                <div class="mt-1 text-3xl font-700">{{ visitorMonth }}</div>
              </div>
              <div
                class="paw-card rounded-[20px] border px-3 py-2"
                :class="isNight
                  ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink'
                  : 'border-meow-line bg-white/70 text-meow-ink'"
              >
                <div class="text-[10px] uppercase tracking-widest" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">总计</div>
                <div class="mt-1 text-3xl font-700">{{ visitorCount }}</div>
              </div>
            </div>
          </div>
        </section>

        <section id="guestbook" class="mt-16">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="font-display text-2xl" :class="isNight ? 'text-meow-night-ink' : ''">互动留言板</h2>
          </div>
          <div
            class="relative mt-4 rounded-3xl p-4 shadow-[0_14px_30px_rgba(47,20,47,0.12)] transition-colors duration-700 ease-in-out motion-card"
            style="--float-delay: 0.25s"
            :class="isNight ? 'bg-meow-night-card/80' : 'bg-white/70'"
          >
            <div id="giscus" class="relative z-1"></div>
          </div>
        </section>

        <footer class="mt-16 text-center text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
          © 2026 Meowhuan. 保持好奇，慢慢来。 ·
          <a
            class="underline underline-offset-4"
            href="https://github.com/meowhuan/Personal_Homepage/"
            target="_blank"
            rel="noreferrer"
          >
          GitHub 
          </a>
        </footer>
      </div>
    </div>
  </div>
</template>

<style>
@keyframes pageFade {
  0% {
    opacity: 0;
  }
  60% {
    opacity: 1;
  }
  100% {
    opacity: 1;
  }
}

.page-fade {
  animation: pageFade 0.8s cubic-bezier(0.22, 1.2, 0.36, 1) both;
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
  background-position: 0 0;
}

.meow-bg.meow-night::before {
  opacity: 0.04;
  background-image: url("data:image/svg+xml;utf8,%3Csvg width='120' height='120' viewBox='0 0 120 120' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%235a6bb8'%3E%3Ccircle cx='24' cy='24' r='6'/%3E%3Ccircle cx='44' cy='18' r='6'/%3E%3Ccircle cx='64' cy='24' r='6'/%3E%3Ccircle cx='32' cy='48' r='12'/%3E%3Ccircle cx='84' cy='84' r='6'/%3E%3Ccircle cx='104' cy='78' r='6'/%3E%3Ccircle cx='96' cy='56' r='6'/%3E%3Ccircle cx='88' cy='100' r='12'/%3E%3C/g%3E%3C/svg%3E");
}

.cord-switch {
  position: fixed;
  top: 4px;
  right: 280px;
  transform: none;
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
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.cord-label {
  height: 8px;
}

.cord-switch:hover .cord-knob {
  transform: translateY(2px);
  box-shadow: 0 10px 20px rgba(47, 20, 47, 0.18);
}

.cord-switch:active .cord-knob {
  transform: translateY(4px);
}

.cord-switch-night .cord-line {
  background: #332b55;
}

.cord-switch-night .cord-knob {
  background: #241f3d;
  color: #f3e9ff;
  border-color: #332b55;
  box-shadow: 0 8px 18px rgba(20, 16, 40, 0.35);
}

.cord-switch-night .cord-label {
  color: #b8a6d8;
}

@media (max-width: 640px) {
  .cord-switch {
    right: 100px;
  }
}

.motion-card {
  transition: transform 0.5s ease, box-shadow 0.5s ease;
}

.motion-card:hover {
  transform: translateY(-6px) scale(1.01);
  box-shadow: 0 16px 32px rgba(47, 20, 47, 0.14);
}

.motion-press {
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.motion-press:hover {
  transform: translateY(-2px);
}

.motion-press:active {
  transform: translateY(0);
}

.nav-link {
  position: relative;
  padding-bottom: 2px;
  transition: color 0.2s ease;
}

.nav-link::after {
  content: "";
  position: absolute;
  left: 0;
  bottom: -2px;
  width: 100%;
  height: 2px;
  transform: scaleX(0);
  transform-origin: left;
  background: linear-gradient(90deg, rgba(255, 173, 214, 0.8), rgba(165, 235, 255, 0.7));
  transition: transform 0.25s ease;
}

.nav-link:hover::after {
  transform: scaleX(1);
}

.meow-window {
  overflow: hidden;
}

.meow-window-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0 10px;
  font-size: 12px;
  letter-spacing: 2px;
  text-transform: uppercase;
  background: transparent;
}

.meow-window-dots {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #f0b1c9;
  box-shadow: 16px 0 0 #f6d48f, 32px 0 0 #b9efdf;
}

.meow-window-title {
  flex: 1;
  font-weight: 600;
  margin-left: 26px;
}

.meow-window-body {
  padding: 0;
  font-size: 12px;
}

.meow-window-item {
  display: grid;
  grid-template-columns: 90px 1fr;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(233, 217, 234, 0.9);
}

.meow-window .meow-window-item {
  color: inherit;
}

.meow-window .meow-window-item .text-meow-night-ink,
.meow-window .meow-window-item .text-meow-ink {
  font-weight: 600;
}

.meow-window.text-meow-night-soft .meow-window-item {
  background: rgba(35, 28, 58, 0.8);
  border-color: rgba(74, 64, 110, 0.8);
}

.meow-window.text-meow-night-soft .meow-window-meta {
  color: rgba(194, 181, 222, 0.75);
}

.meow-window-time {
  font-weight: 700;
  letter-spacing: 1px;
}

.meow-window-titleline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-weight: 600;
}

.meow-window-meta {
  margin-top: 4px;
  color: rgba(90, 76, 95, 0.8);
}

.meow-window-note {
  margin-top: 6px;
  line-height: 1.5;
}

.status-list {
  max-height: 96px;
  overflow: hidden;
}

.quick-card {
  display: flex;
  flex-direction: column;
  max-height: 520px;
}

.quick-card-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  overflow-y: auto;
  padding-right: 4px;
}

.meow-title {
  font-weight: 700;
  letter-spacing: 2px;
}

.meow-pill-strong {
  padding: 2px 10px;
  background: linear-gradient(90deg, rgba(201, 170, 236, 0.9), rgba(164, 210, 255, 0.9));
  color: #2b1d2a;
  border: 0;
  box-shadow: 0 8px 16px rgba(90, 60, 120, 0.2);
  transform-origin: center;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.meow-pill-strong:hover {
  transform: translateY(-1px) scale(1.02);
  box-shadow: 0 10px 20px rgba(90, 60, 120, 0.25);
}

.paw-card {
  position: relative;
  background: linear-gradient(140deg, rgba(255, 220, 236, 0.8), rgba(220, 230, 255, 0.75));
  backdrop-filter: blur(10px);
}

.paw-card > * {
  position: relative;
  z-index: 1;
}

.meow-night .paw-card {
  background: linear-gradient(140deg, rgba(120, 140, 230, 0.35), rgba(84, 110, 200, 0.35));
}

.paw-card::before {
  content: "";
  position: absolute;
  bottom: 6px;
  right: 6px;
  width: 26px;
  height: 22px;
  opacity: 0.35;
  z-index: 0;
  background-repeat: no-repeat;
  background-size: contain;
  background-image: url("data:image/svg+xml;utf8,%3Csvg width='68' height='56' viewBox='0 0 68 56' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%23ffffff'%3E%3Ccircle cx='18' cy='16' r='6'/%3E%3Ccircle cx='34' cy='12' r='6'/%3E%3Ccircle cx='50' cy='16' r='6'/%3E%3Ccircle cx='30' cy='38' r='12'/%3E%3C/g%3E%3C/svg%3E");
}

.meow-night .paw-card::before {
  opacity: 0.3;
  background-image: url("data:image/svg+xml;utf8,%3Csvg width='68' height='56' viewBox='0 0 68 56' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%23d7e2ff'%3E%3Ccircle cx='18' cy='16' r='6'/%3E%3Ccircle cx='34' cy='12' r='6'/%3E%3Ccircle cx='50' cy='16' r='6'/%3E%3Ccircle cx='30' cy='38' r='12'/%3E%3C/g%3E%3C/svg%3E");
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

.status-dot-error {
  background: rgba(235, 86, 120, 0.9);
  box-shadow: 0 0 0 4px rgba(235, 86, 120, 0.18);
}

.meow-sad {
  width: 22px;
  height: 22px;
  color: rgba(235, 86, 120, 0.9);
}

@media (max-width: 640px) {
  .paw-card::before {
    bottom: 4px;
    right: 4px;
    width: 22px;
    height: 18px;
  }
}


@media (max-width: 640px) {
  .quick-card {
    max-height: none;
  }

  .quick-card-body {
    overflow: visible;
    padding-right: 0;
  }

  .meow-window-item {
    grid-template-columns: 1fr;
  }

  .meow-window-titleline {
    flex-direction: column;
    align-items: flex-start;
  }
}

@keyframes floatSoft {
  0% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-6px);
  }
  100% {
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .motion-card {
    animation: none;
  }

  .motion-card:hover,
  .motion-press:hover,
  .motion-press:active {
    transform: none;
  }

  .nav-link::after {
    transition: none;
  }
}

</style>
