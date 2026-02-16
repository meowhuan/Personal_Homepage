<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const isNight = ref(false);
const currentPost = ref("");
const blogLoading = ref(false);
const blogError = ref(false);
const detailLoading = ref(false);

const fallbackPosts = [
  {
    slug: "welcome-to-blog",
    title: "博客开张：给自己留一个角落",
    date: "2026-02-16",
    tag: "日常",
    excerpt: "开始认真记录，不追求高产，只希望留下真实的痕迹。",
    content: [
      "这个页面是新开的博客入口。想把零散的想法、日常和项目笔记都放在这里，慢慢积累。",
      "后续会继续加文章分类、按时间归档，也会把一些项目踩坑写得更详细。"
    ]
  },
  {
    slug: "homepage-notes",
    title: "主页更新日志：视觉和结构的几次调整",
    date: "2026-02-14",
    tag: "开发",
    excerpt: "记录主页最近的设计变化，方便回看和持续优化。",
    content: [
      "这版主页重点是保持轻松感：柔和背景、圆角卡片、低对比边框，以及更明确的信息分区。",
      "后续会继续做移动端细节和加载性能优化，让页面更顺滑。"
    ]
  },
  {
    slug: "small-things",
    title: "一些小事：奶茶、代码和慢慢来",
    date: "2026-02-10",
    tag: "碎碎念",
    excerpt: "不那么正式的一篇，写写最近的状态。",
    content: [
      "最近节奏还是偏慢，但也在一点点推进。能把事情做完，比一下子做很多更重要。",
      "博客会持续更新，频率随缘，但会尽量保持真实。"
    ]
  }
];

const blogApiBase = "https://m.ratf.cn/blog";
const posts = ref([]);
const activePost = ref(null);

const isListView = computed(() => !currentPost.value);
const markdownImagePattern = /^!\[(.*?)\]\((.+?)\)$/;
const imageExtPattern = /\.(png|jpe?g|gif|webp|avif|svg)(\?.*)?$/i;

const formatDate = (dateStr) =>
  new Date(dateStr).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit"
  });

const splitTags = (tagValue) => {
  if (!tagValue || typeof tagValue !== "string") return [];
  return tagValue
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);
};

const getPostTags = (post) => {
  const tags = splitTags(post?.tag);
  return tags.length ? tags : ["博客"];
};

const isLikelyImageUrl = (value) => {
  if (!value) return false;
  try {
    const url = new URL(value);
    if (!["http:", "https:"].includes(url.protocol)) return false;
    return imageExtPattern.test(`${url.pathname}${url.search}`);
  } catch {
    return false;
  }
};

const normalizeImageBlock = (url, alt, idx) => ({
  type: "image",
  key: `img-${idx}-${url}`,
  url,
  alt: alt?.trim() || "博客图片"
});

const activeContentBlocks = computed(() => {
  const lines = Array.isArray(activePost.value?.content) ? activePost.value.content : [];
  return lines
    .map((raw, idx) => {
      const text = String(raw || "").trim();
      if (!text) return null;
      const markdownMatched = text.match(markdownImagePattern);
      if (markdownMatched) {
        return normalizeImageBlock(markdownMatched[2], markdownMatched[1], idx);
      }
      if (isLikelyImageUrl(text)) {
        return normalizeImageBlock(text, "", idx);
      }
      return { type: "text", key: `txt-${idx}`, text };
    })
    .filter(Boolean);
});

const readPostFromQuery = () => {
  const url = new URL(window.location.href);
  const slug = url.searchParams.get("post") || "";
  currentPost.value = slug;
};

const openPost = (slug) => {
  const url = new URL(window.location.href);
  url.searchParams.set("post", slug);
  window.history.pushState({}, "", `${url.pathname}${url.search}`);
  currentPost.value = slug;
  fetchBlogDetail(slug);
};

const backToList = () => {
  const url = new URL(window.location.href);
  url.searchParams.delete("post");
  window.history.pushState({}, "", `${url.pathname}${url.search}`);
  currentPost.value = "";
  activePost.value = null;
};

const toggleTheme = () => {
  isNight.value = !isNight.value;
  localStorage.setItem("meow-theme", isNight.value ? "night" : "day");
};

const fetchBlogList = async () => {
  blogLoading.value = true;
  try {
    const res = await fetch(blogApiBase);
    if (!res.ok) throw new Error("blog list fetch failed");
    const data = await res.json();
    if (!Array.isArray(data)) throw new Error("blog list invalid");
    posts.value = data;
    blogError.value = false;
  } catch {
    posts.value = fallbackPosts.map(({ content, ...item }) => item);
    blogError.value = true;
  } finally {
    blogLoading.value = false;
  }
};

const fetchBlogDetail = async (slug) => {
  if (!slug) {
    activePost.value = null;
    return;
  }
  detailLoading.value = true;
  try {
    const res = await fetch(`${blogApiBase}/${encodeURIComponent(slug)}`);
    if (!res.ok) throw new Error("blog detail fetch failed");
    const data = await res.json();
    activePost.value = data;
  } catch {
    activePost.value = fallbackPosts.find((post) => post.slug === slug) || null;
  } finally {
    detailLoading.value = false;
  }
};

const onPopState = async () => {
  readPostFromQuery();
  await fetchBlogDetail(currentPost.value);
};

onMounted(() => {
  const savedTheme = localStorage.getItem("meow-theme");
  if (savedTheme) {
    isNight.value = savedTheme === "night";
  } else if (window.matchMedia) {
    isNight.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  readPostFromQuery();
  fetchBlogList().then(() => fetchBlogDetail(currentPost.value));
  window.addEventListener("popstate", onPopState);
});

onBeforeUnmount(() => {
  window.removeEventListener("popstate", onPopState);
});
</script>

<template>
  <div
    class="min-h-screen font-body transition-colors duration-700 ease-in-out meow-bg"
    :class="isNight
      ? 'bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink meow-night'
      : 'bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink meow-day'"
  >
    <main class="relative mx-auto w-[min(960px,92vw)] py-10">
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
          <h1 class="font-display text-3xl sm:text-4xl">喵喵博客</h1>
          <p class="mt-2 text-sm" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
            记录日常、项目和一些临时冒出来的想法。
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
            href="/donate.html"
            class="meow-btn-ghost"
            :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
          >
            喵喵补给站
          </a>
        </div>
      </div>

      <section v-if="isListView" class="mt-8 grid gap-4">
        <div
          v-if="blogLoading"
          class="text-sm"
          :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
        >
          博客加载中...
        </div>
        <div
          v-if="blogError"
          class="text-xs"
          :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
        >
          后端暂不可用，已显示本地缓存内容。
        </div>
        <article
          v-for="post in posts"
          :key="post.slug"
          class="meow-card motion-card rounded-3xl p-5"
          :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
        >
          <div class="flex flex-wrap items-center gap-2">
            <span
              v-for="tag in getPostTags(post)"
              :key="`${post.slug}-${tag}`"
              class="meow-pill"
            >
              {{ tag }}
            </span>
            <span class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">{{ formatDate(post.date) }}</span>
          </div>
          <h2 class="mt-3 font-display text-2xl">{{ post.title }}</h2>
          <p class="mt-3 text-sm leading-relaxed" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
            {{ post.excerpt }}
          </p>
          <button
            class="meow-btn-primary motion-press mt-4"
            :class="isNight ? 'bg-meow-night-accent text-meow-night-bg' : ''"
            type="button"
            @click="openPost(post.slug)"
          >
            阅读全文
          </button>
        </article>
      </section>

      <section
        v-else-if="activePost"
        class="meow-card mt-8 rounded-3xl p-6"
        :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
      >
        <button
          class="meow-btn-ghost"
          :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-bg/80' : ''"
          type="button"
          @click="backToList"
        >
          返回列表
        </button>
        <div class="mt-4 flex flex-wrap items-center gap-2">
          <span
            v-for="tag in getPostTags(activePost)"
            :key="`${activePost.slug}-${tag}`"
            class="meow-pill"
          >
            {{ tag }}
          </span>
          <span class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">{{ formatDate(activePost.date) }}</span>
        </div>
        <h2 class="mt-4 font-display text-3xl">{{ activePost.title }}</h2>
        <div class="mt-5 space-y-4">
          <p
            v-if="detailLoading"
            class="text-sm"
            :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
          >
            正在加载全文...
          </p>
          <template v-for="block in activeContentBlocks" :key="block.key">
            <p
              v-if="block.type === 'text'"
              class="text-sm leading-relaxed"
              :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
            >
              {{ block.text }}
            </p>
            <figure v-else class="blog-figure" :class="isNight ? 'blog-figure-night' : ''">
              <img
                :src="block.url"
                :alt="block.alt"
                loading="lazy"
                decoding="async"
                referrerpolicy="no-referrer"
                class="blog-image"
              />
              <figcaption
                v-if="block.alt && block.alt !== '博客图片'"
                class="blog-caption"
                :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
              >
                {{ block.alt }}
              </figcaption>
            </figure>
          </template>
        </div>
      </section>

      <section
        v-else
        class="meow-card mt-8 rounded-3xl p-6 text-sm"
        :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
      >
        没有找到这篇文章，返回列表看看其他内容吧。
      </section>
    </main>
  </div>
</template>

<style>
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

.motion-press:active {
  transform: translateY(0);
}

.blog-figure {
  margin: 0;
  overflow: hidden;
  border: 1px solid rgba(233, 217, 234, 0.9);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.72);
  box-shadow: 0 12px 28px rgba(47, 20, 47, 0.1);
}

.blog-figure-night {
  border-color: rgba(74, 64, 110, 0.82);
  background: rgba(35, 28, 58, 0.82);
}

.blog-image {
  display: block;
  width: 100%;
  max-height: min(68vh, 560px);
  object-fit: contain;
  background: rgba(255, 255, 255, 0.45);
}

.blog-caption {
  padding: 8px 12px 10px;
  font-size: 12px;
  line-height: 1.5;
}
</style>
