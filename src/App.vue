<script setup>
import { useHomepageState } from "./composables/useHomepageState";

const {
  time,
  date,
  calendar,
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
} = useHomepageState();
</script>
<template>
  <div
    class="min-h-screen font-body page-fade transition-colors duration-700 ease-in-out meow-bg"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink meow-night'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink meow-day'"
  >
    <Transition name="intro-fade">
      <div
        v-if="showIntro"
        class="intro-loader"
        :class="isNight ? 'intro-loader-night' : 'intro-loader-day'"
        aria-hidden="true"
      >
        <div class="intro-loader-inner">
          <span class="intro-dot"></span>
          <span class="intro-dot"></span>
          <span class="intro-dot"></span>
        </div>
      </div>
    </Transition>
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
            <div class="flex items-center gap-2">
              <button
                class="meow-pill motion-press px-2 py-0.5 text-[11px]"
                type="button"
                :class="[
                  (!canFetchBlog() || blogLoading) ? 'opacity-50' : '',
                  isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''
                ]"
                :disabled="blogLoading || !canFetchBlog()"
                @click="fetchBlog()"
              >
                {{ blogLoading ? "刷新中" : (canFetchBlog() ? "刷新" : "冷却中") }}
              </button>
              <a
                class="meow-pill motion-press"
                :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
                href="/blog.html"
              >
                查看全部
              </a>
            </div>
          </div>
          <div class="mt-2 text-[11px]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'" v-if="blogUpdatedAt">
            更新于 {{ new Date(blogUpdatedAt).toLocaleTimeString("zh-CN") }}
          </div>
          <div class="mt-2 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'" v-if="blogError && blogList.length === 0">
            暂时无法获取博客更新
          </div>
          <div class="mt-2 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'" v-else-if="!blogLoading && blogList.length === 0">
            暂无博客内容
          </div>
          <div class="mt-6 grid gap-4 md:grid-cols-3">
            <article
              v-for="(post, idx) in blogList.slice(0, 3)"
              :key="post.slug"
              class="meow-card motion-card p-5"
              :style="{ '--float-delay': `${0.1 + idx * 0.25}s` }"
              :class="isNight ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
            >
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="tag in (splitTags(post.tag).length ? splitTags(post.tag) : ['博客'])"
                  :key="`${post.slug}-${tag}`"
                  class="meow-pill"
                >
                  {{ tag }}
                </span>
              </div>
              <h3 class="mt-3 text-base font-600">{{ post.title }}</h3>
              <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                {{ post.excerpt || "暂无摘要" }}
              </p>
              <a
                class="meow-pill motion-press mt-4 inline-flex"
                :href="`/blog.html?post=${encodeURIComponent(post.slug)}`"
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

<style src="./styles/homepage.css"></style>
