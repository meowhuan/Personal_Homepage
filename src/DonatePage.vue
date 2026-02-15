<script setup>
import { computed, onMounted, ref } from "vue";

const isNight = ref(false);

onMounted(() => {
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) {
    isNight.value = savedTheme === "night";
    return;
  }
  if (window.matchMedia) {
    isNight.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
});

const cardClass = computed(() =>
  isNight.value
    ? "bg-meow-night-card/85 border-meow-night-line text-meow-night-soft"
    : "bg-white/75 border-meow-line text-meow-soft"
);

const titleClass = computed(() => (isNight.value ? "text-meow-night-ink" : "text-meow-ink"));
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

      <section class="mt-8 grid gap-4 md:grid-cols-2">
        <article class="rounded-3xl border p-5 shadow-[0_12px_28px_rgba(47,20,47,0.1)]" :class="cardClass">
          <h2 class="text-lg font-700" :class="titleClass">微信投喂</h2>
          <p class="mt-2 text-sm leading-relaxed">
            轻轻一扫，把温柔传送给 meow。谢谢你愿意支持这个小角落。
          </p>
          <img
            class="mt-4 h-[320px] w-full rounded-2xl border bg-white p-2 object-contain md:h-[360px]"
            :class="isNight ? 'border-meow-night-line' : 'border-meow-line'"
            src="https://image.meowra.cn/i/2026/02/15/6991426216e96.png"
            alt="微信赞赏码"
          />
        </article>

        <article class="rounded-3xl border p-5 shadow-[0_12px_28px_rgba(47,20,47,0.1)]" :class="cardClass">
          <h2 class="text-lg font-700" :class="titleClass">支付宝投喂</h2>
          <p class="mt-2 text-sm leading-relaxed">
            这份赞赏我会认真收好，拿去喂养灵感、代码和一点点日常勇气。
          </p>
          <img
            class="mt-4 h-[320px] w-full rounded-2xl border bg-white p-2 object-contain md:h-[360px]"
            :class="isNight ? 'border-meow-night-line' : 'border-meow-line'"
            src="https://image.meowra.cn/i/2026/02/15/69914263c15c8.jpg"
            alt="支付宝赞赏码"
          />
        </article>
      </section>
    </main>
  </div>
</template>
