<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from "vue";

const time = ref("");
const date = ref("");
const calendar = ref({ day: "", month: "", weekday: "" });
let timer = 0;
const isNight = ref(false);
const feedCount = ref(0);
const hearts = ref([]);
let heartId = 0;
const quoteText = ref("加载中…");
const quoteFrom = ref("");
const quoteError = ref(false);
const quoteLoading = ref(false);
const nextQuoteAt = ref(0);
const cooldownMs = 1500;

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
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) isNight.value = savedTheme === "night";
  const savedFeed = localStorage.getItem("meow-feed-count");
  if (savedFeed) feedCount.value = Number(savedFeed);
  loadGiscus();
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

const toggleTheme = () => {
  isNight.value = !isNight.value;
  localStorage.setItem("meow-theme", isNight.value ? "night" : "day");
};

const persistFeed = () => {
  localStorage.setItem("meow-feed-count", String(feedCount.value));
};

const feedCat = () => {
  feedCount.value += 1;
  persistFeed();
  const id = heartId++;
  hearts.value.push(id);
  setTimeout(() => {
    hearts.value = hearts.value.filter((h) => h !== id);
  }, 1400);
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
      ? `${location.origin}/giscus-dark.css`
      : `${location.origin}/giscus.css`
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
            ? `${location.origin}/giscus-dark.css`
            : `${location.origin}/giscus.css`
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
    class="min-h-screen font-body page-fade transition-colors duration-500"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink'"
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
            <a :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#about">关于我</a>
            <a :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#stuff">我在做</a>
            <a :class="isNight ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'" href="#contact">联系</a>
          </div>
        </nav>

        <div class="mt-6 grid gap-4 md:grid-cols-[1fr_auto] md:items-stretch">
          <div
            class="meow-card flex h-full min-h-[96px] items-start gap-3 rounded-3xl px-5 py-4 text-xs backdrop-blur"
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
              class="meow-pill px-2 py-1 text-[11px] transition-opacity"
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
            class="meow-card flex h-full min-h-[96px] items-center gap-4 rounded-3xl px-5 py-4 text-xs backdrop-blur md:justify-self-end"
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
                class="meow-btn-primary"
                :class="isNight ? 'bg-meow-night-accent text-meow-night-bg' : ''"
                href="mailto:meowhuan@qq.com"
              >
                Email me
              </a>
              <a
                class="meow-btn-ghost"
                :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                href="https://github.com/meowhuan"
                target="_blank"
                rel="noreferrer"
              >
                GitHub
              </a>
              <a
                class="meow-btn-ghost"
                :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
                href="https://x.com/meow_huan"
                target="_blank"
                rel="noreferrer"
              >
                X
              </a>
            </div>
          </div>

          <div
            class="meow-card p-6"
            :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
          >
            <h2 class="font-display text-2xl">快速了解我</h2>
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
          </div>
        </section>

        <section id="about" class="mt-16">
          <h2 class="font-display text-2xl">关于我</h2>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">我在意的事</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                并没有在意的事 ( •̥́ ˍ •̀ू )，只想过好每一天，和喜欢的人一起做喜欢的事。
              </p>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">我的兴趣</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                编程、记录、睡觉、奶茶、明日方舟，还有可爱的设计和配色。
              </p>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
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
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">Oyama's HRT Tracker</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                为记录与跟踪 HRT 过程的前端小工具编写的后端程序，基本完善，不定期维护。
              </p>
              <a
                class="meow-pill mt-4 inline-flex"
                href="https://github.com/meowhuan/Oyama-s-HRT-Tracker"
                target="_blank"
                rel="noreferrer"
              >
                查看项目
              </a>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">技能积累</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                随缘学习新技能，但是不知道学什么好，感觉什么都想学又什么都学不好。
              </p>
              <span class="meow-pill mt-4">学习中</span>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">小记录</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                偶尔记录一些日常小事，想记录但又不想记录，感觉有点矛盾 ( •̥́ ˍ •̀ू )。
              </p>
              <span class="meow-pill mt-4">随缘更新</span>
            </article>
          </div>
        </section>

        <section id="contact" class="mt-16">
          <h2 class="font-display text-2xl">联系我</h2>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">邮箱</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">meowhuan@qq.com</p>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">GitHub</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">github.com/meowhuan</p>
            </article>
            <article class="meow-card p-5" :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
              <h3 class="text-base font-600">X</h3>
              <p class="mt-3 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">x.com/meow_huan</p>
            </article>
          </div>
        </section>

        <section id="guestbook" class="mt-16">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="font-display text-2xl" :class="isNight ? 'text-meow-night-ink' : ''">互动留言板</h2>
            <div class="flex items-center gap-2">
              <button
                class="meow-btn-primary px-4 py-2 text-xs"
                :class="isNight ? 'bg-meow-night-accent text-meow-night-bg' : ''"
                type="button"
                @click="feedCat"
              >
                喂小猫 +1
              </button>
              <span class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                已喂食 {{ feedCount }} 次
              </span>
            </div>
          </div>
          <div
            class="relative mt-4 overflow-hidden rounded-3xl p-4 shadow-[0_14px_30px_rgba(47,20,47,0.12)]"
            :class="isNight ? 'bg-meow-night-card/80' : 'bg-white/70'"
          >
            <div class="pointer-events-none absolute inset-0">
              <span
                v-for="id in hearts"
                :key="id"
                class="absolute bottom-4 left-6 animate-heart"
              >
                💗
              </span>
            </div>
            <div id="giscus" class="relative z-1"></div>
          </div>
        </section>

        <footer class="mt-16 text-center text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
          © 2026 Meowhuan. 保持好奇，慢慢来。
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

@keyframes heartFloat {
  0% {
    opacity: 0;
    transform: translateY(0) scale(0.6);
  }
  30% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translateY(-80px) scale(1.2);
  }
}

.animate-heart {
  animation: heartFloat 1.4s ease forwards;
}
</style>
