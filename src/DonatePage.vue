<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const isNight = ref(false);
const themeMedia = ref(null);

const onSystemThemeChange = (event) => {
  if (localStorage.getItem("meow-theme")) return;
  isNight.value = event.matches;
};

onMounted(() => {
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) {
    isNight.value = savedTheme === "night";
    return;
  }
  if (window.matchMedia) {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    isNight.value = media.matches;
    themeMedia.value = media;
    media.addEventListener("change", onSystemThemeChange);
  }
});

onBeforeUnmount(() => {
  if (themeMedia.value) {
    themeMedia.value.removeEventListener("change", onSystemThemeChange);
  }
});

const cardClass = computed(() =>
  isNight.value
    ? "bg-meow-night-card/85 border-meow-night-line text-meow-night-soft"
    : "bg-white/75 border-meow-line text-meow-soft"
);

const titleClass = computed(() => (isNight.value ? "text-meow-night-ink" : "text-meow-ink"));

const gratitudeList = ref([
  {
    date: "2026-03-17 22:52:04",
    amount: 162,
    name: "姐姐",
    message: "世界最可爱的小天使，每天都要开心哦",
  },
]);
</script>

<template>
  <div
    class="min-h-screen transition-colors duration-700 ease-in-out"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink'"
  >
    <main class="mx-auto w-[min(920px,92vw)] py-10">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <h1 class="font-display text-3xl sm:text-4xl">喵喵补给站</h1>
        <a
          href="/"
          class="meow-btn-ghost"
          :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
        >
          返回主页
        </a>
      </div>

      <p class="mt-4 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
        如果这里的内容让你觉得有一点点被陪伴到，欢迎给小猫投喂一杯奶茶。你的每一份心意，都会变成我继续更新和维护站点的动力。
      </p>

      <section class="mt-8">
        <article class="rounded-3xl border p-6 shadow-[0_12px_28px_rgba(47,20,47,0.1)]" :class="cardClass">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="text-lg font-700" :class="titleClass">爱发电 · 首选</h2>
            <span
              class="rounded-full border px-3 py-1 text-xs tracking-wide"
              :class="isNight ? 'border-meow-night-line text-meow-night-soft' : 'border-meow-line text-meow-soft'"
            >
              优先支持
            </span>
          </div>
          <p class="mt-3 text-sm leading-relaxed">
            如果方便的话，欢迎优先使用爱发电来投喂 meow。你的支持会直接变成更多内容与更新的动力。
          </p>
          <a
            class="meow-btn mt-4 inline-flex items-center justify-center px-6 py-2.5 text-sm transition-transform duration-200 hover:-translate-y-0.5"
            :class="isNight
              ? 'shadow-[0_6px_14px_rgba(90,120,180,0.2)]'
              : 'shadow-[0_6px_14px_rgba(47,20,47,0.12)]'"
            href="https://ifdian.net/a/meowhuan"
            target="_blank"
            rel="noopener noreferrer"
          >
            去爱发电主页
          </a>
        </article>
      </section>

      <section class="mt-6 grid gap-4 md:grid-cols-2">
        <article class="rounded-3xl border p-5 shadow-[0_12px_28px_rgba(47,20,47,0.1)] md:col-span-2" :class="cardClass">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="text-lg font-700" :class="titleClass">感谢名单</h2>
            <span
              class="rounded-full border px-3 py-1 text-xs tracking-wide"
              :class="isNight ? 'border-meow-night-line text-meow-night-soft' : 'border-meow-line text-meow-soft'"
            >
              按更新顺序展示
            </span>
          </div>
          <p class="mt-2 text-sm leading-relaxed">
            感谢每一位赞助和鼓励我的朋友。这里会持续更新公开署名的喵友名单。
          </p>
          <ul v-if="gratitudeList.length" class="mt-4 grid gap-2 sm:grid-cols-2">
            <li
              v-for="(item, index) in gratitudeList"
              :key="`${item.name}-${index}`"
              class="rounded-2xl border px-3 py-2 text-sm"
              :class="isNight ? 'border-meow-night-line/80 bg-meow-night-card/65' : 'border-meow-line/80 bg-white/60'"
            >
              <div class="font-600" :class="titleClass">{{ index + 1 }}. {{ item.name }}</div>
              <div class="mt-1 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                {{ item.date }} · 爱发电赞助 {{ item.amount }} 元
              </div>
              <div class="mt-1 text-sm">留言：{{ item.message }}</div>
            </li>
          </ul>
          <div
            v-else
            class="mt-4 rounded-2xl border border-dashed px-4 py-3 text-sm"
            :class="isNight ? 'border-meow-night-line text-meow-night-soft' : 'border-meow-line text-meow-soft'"
          >
            名单整理中，公开署名后会第一时间更新在这里。
          </div>
        </article>

        <article class="rounded-3xl border p-5 shadow-[0_12px_28px_rgba(47,20,47,0.1)]" :class="cardClass">
          <h2 class="text-lg font-700" :class="titleClass">想对你说</h2>
          <p class="mt-2 text-sm leading-relaxed">
            你的支持让我更有勇气继续更新这个小角落。无论是赞助、鼓励，还是来打个招呼，都会让我很开心。
          </p>
          <div class="mt-4 text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
            谢谢你愿意在这里停留一下。
          </div>
        </article>

        <article class="rounded-3xl border p-5 shadow-[0_12px_28px_rgba(47,20,47,0.1)]" :class="cardClass">
          <h2 class="text-lg font-700" :class="titleClass">其他方式请私信</h2>
          <p class="mt-2 text-sm leading-relaxed">
            为了保护隐私，其他赞助方式不在页面公开。如果你需要，请通过主页的联系方式私信我，我会告诉你。
          </p>
          <a
            class="meow-btn-ghost mt-4 inline-flex items-center justify-center"
            :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
            href="/#contact"
          >
            前往联系方式
          </a>
        </article>
      </section>

    </main>
  </div>
</template>
