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
const searchQuery = ref("");
const selectedTags = ref([]);
const tagMenuOpen = ref(false);
const tagMenuRef = ref(null);
const themeMedia = ref(null);

const isListView = computed(() => !currentPost.value);

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

const availableTags = computed(() => {
  const allTags = posts.value.flatMap((post) => getPostTags(post));
  return [...new Set(allTags)];
});

const normalizeForSearch = (value) => String(value || "").trim().toLowerCase();

const matchesSearch = (post) => {
  const keyword = normalizeForSearch(searchQuery.value);
  if (!keyword) return true;
  const haystack = [
    post.title,
    post.excerpt,
    ...getPostTags(post)
  ]
    .map(normalizeForSearch)
    .join(" ");
  return haystack.includes(keyword);
};

const matchesTags = (post) => {
  if (selectedTags.value.length === 0) return true;
  const tags = getPostTags(post);
  return selectedTags.value.some((tag) => tags.includes(tag));
};

const filteredPosts = computed(() =>
  posts.value.filter((post) => matchesSearch(post) && matchesTags(post))
);

const toggleTagFilter = (tag) => {
  if (selectedTags.value.includes(tag)) {
    selectedTags.value = selectedTags.value.filter((item) => item !== tag);
    return;
  }
  selectedTags.value = [...selectedTags.value, tag];
};

const clearTagFilters = () => {
  selectedTags.value = [];
};

const clearFilters = () => {
  searchQuery.value = "";
  selectedTags.value = [];
};

const toggleTagMenu = () => {
  tagMenuOpen.value = !tagMenuOpen.value;
};

const onDocumentClick = (event) => {
  if (!tagMenuOpen.value) return;
  const container = tagMenuRef.value;
  if (!container) return;
  if (!container.contains(event.target)) {
    tagMenuOpen.value = false;
  }
};

const escapeHtml = (value) =>
  String(value || "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");

const safeUrl = (value) => {
  const raw = String(value || "").trim();
  if (!raw) return "";
  if (raw.startsWith("/") || raw.startsWith("./") || raw.startsWith("../") || raw.startsWith("#")) {
    return raw;
  }
  try {
    const parsed = new URL(raw);
    return ["http:", "https:", "mailto:"].includes(parsed.protocol) ? raw : "";
  } catch {
    return "";
  }
};

const renderInlineMarkdown = (text) => {
  let html = escapeHtml(text);
  html = html.replace(/`([^`]+)`/g, (_, code) => `<code>${code}</code>`);
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  html = html.replace(/~~([^~]+)~~/g, "<del>$1</del>");
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, url) => {
    const safe = safeUrl(url);
    if (!safe) return "";
    const altText = escapeHtml(alt || "博客图片");
    return `<img src="${escapeHtml(safe)}" alt="${altText}" loading="lazy" decoding="async" referrerpolicy="no-referrer" class="blog-image" />`;
  });
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, label, url) => {
    const safe = safeUrl(url);
    if (!safe) return escapeHtml(label);
    return `<a href="${escapeHtml(safe)}" target="_blank" rel="noreferrer noopener">${escapeHtml(label)}</a>`;
  });
  return html;
};

const renderMarkdown = (markdown) => {
  const src = String(markdown || "").replaceAll("\r\n", "\n");
  const lines = src.split("\n");
  const html = [];
  let inCode = false;
  let codeLines = [];
  let listType = "";
  let tableHead = null;
  let tableRows = [];

  const closeList = () => {
    if (!listType) return;
    html.push(listType === "ol" ? "</ol>" : "</ul>");
    listType = "";
  };
  const flushTable = () => {
    if (!tableHead) return;
    const headHtml = `<thead><tr>${tableHead.map((cell) => `<th>${renderInlineMarkdown(cell)}</th>`).join("")}</tr></thead>`;
    const bodyHtml = tableRows.length
      ? `<tbody>${tableRows.map((row) => `<tr>${row.map((cell) => `<td>${renderInlineMarkdown(cell)}</td>`).join("")}</tr>`).join("")}</tbody>`
      : "";
    html.push(`<table>${headHtml}${bodyHtml}</table>`);
    tableHead = null;
    tableRows = [];
  };

  for (let i = 0; i < lines.length; i += 1) {
    const rawLine = lines[i];
    const line = String(rawLine || "");
    const trim = line.trim();
    if (trim.startsWith("```")) {
      closeList();
      flushTable();
      if (!inCode) {
        inCode = true;
        codeLines = [];
      } else {
        html.push(`<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
        inCode = false;
      }
      continue;
    }
    if (inCode) {
      codeLines.push(line);
      continue;
    }
    if (!trim) {
      closeList();
      flushTable();
      continue;
    }
    const heading = trim.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      closeList();
      flushTable();
      const level = heading[1].length;
      html.push(`<h${level}>${renderInlineMarkdown(heading[2])}</h${level}>`);
      continue;
    }
    if (trim === "---" || trim === "***" || trim === "___") {
      closeList();
      flushTable();
      html.push("<hr />");
      continue;
    }
    const ul = trim.match(/^[-*+]\s+(.+)$/);
    if (ul) {
      flushTable();
      if (listType !== "ul") {
        closeList();
        html.push("<ul>");
        listType = "ul";
      }
      const task = ul[1].match(/^\[( |x|X)\]\s+(.+)$/);
      if (task) {
        const checked = task[1].toLowerCase() === "x";
        html.push(`<li><input type="checkbox" disabled ${checked ? "checked" : ""} /> ${renderInlineMarkdown(task[2])}</li>`);
      } else {
        html.push(`<li>${renderInlineMarkdown(ul[1])}</li>`);
      }
      continue;
    }
    const ol = trim.match(/^\d+\.\s+(.+)$/);
    if (ol) {
      flushTable();
      if (listType !== "ol") {
        closeList();
        html.push("<ol>");
        listType = "ol";
      }
      html.push(`<li>${renderInlineMarkdown(ol[1])}</li>`);
      continue;
    }
    if (trim.startsWith(">")) {
      closeList();
      flushTable();
      html.push(`<blockquote>${renderInlineMarkdown(trim.replace(/^>\s?/, ""))}</blockquote>`);
      continue;
    }

    const isTableLine = trim.includes("|");
    if (isTableLine) {
      const cells = trim
        .split("|")
        .map((v) => v.trim())
        .filter((_, idx, arr) => !(idx === 0 && arr[idx] === "") && !(idx === arr.length - 1 && arr[idx] === ""));
      const next = String(lines[i + 1] || "").trim();
      const isAlignLine = /^[:\-|\s]+$/.test(next) && next.includes("-");
      if (!tableHead && cells.length > 0 && isAlignLine) {
        closeList();
        tableHead = cells;
        i += 1;
        continue;
      }
      if (tableHead && cells.length > 0) {
        closeList();
        tableRows.push(cells);
        continue;
      }
    }

    closeList();
    flushTable();
    html.push(`<p>${renderInlineMarkdown(trim)}</p>`);
  }

  if (inCode) {
    html.push(`<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
  }
  closeList();
  flushTable();
  return html.join("");
};

const activeContentHtml = computed(() => {
  if (!activePost.value) return "";
  const markdown =
    typeof activePost.value.content_md === "string"
      ? activePost.value.content_md
      : Array.isArray(activePost.value.content)
        ? activePost.value.content.join("\n")
        : "";
  return renderMarkdown(markdown);
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
  if (themeMedia.value) {
    themeMedia.value.removeEventListener("change", onSystemThemeChange);
    themeMedia.value = null;
  }
};

const onSystemThemeChange = (event) => {
  if (localStorage.getItem("meow-theme")) return;
  isNight.value = event.matches;
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
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    isNight.value = media.matches;
    themeMedia.value = media;
    media.addEventListener("change", onSystemThemeChange);
  }
  readPostFromQuery();
  fetchBlogList().then(() => fetchBlogDetail(currentPost.value));
  window.addEventListener("popstate", onPopState);
  document.addEventListener("click", onDocumentClick);
});

onBeforeUnmount(() => {
  window.removeEventListener("popstate", onPopState);
  document.removeEventListener("click", onDocumentClick);
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
          <Transition name="top-action-fade">
            <button
              v-if="!isListView"
              type="button"
              class="meow-btn-ghost"
              :class="isNight ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''"
              @click="backToList"
            >
              返回博客列表
            </button>
          </Transition>
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

      <Transition name="blog-switch" mode="out-in">
      <section v-if="isListView" class="mt-8 grid gap-4" key="blog-list">
        <div
          class="meow-card rounded-3xl p-4"
          :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
        >
          <div class="grid gap-3 md:grid-cols-[1fr_auto] md:items-center">
            <input
              v-model.trim="searchQuery"
              type="text"
              placeholder="搜索文章（标题 / 摘要 / 标签）"
              class="w-full rounded-2xl border px-4 py-2 text-sm outline-none transition-shadow"
              :class="isNight
                ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink placeholder:text-meow-night-soft/80 focus:shadow-[0_0_0_3px_rgba(136,243,255,0.18)]'
                : 'border-meow-line bg-white/80 text-meow-ink placeholder:text-meow-soft focus:shadow-[0_0_0_3px_rgba(255,122,182,0.22)]'"
            />
            <div class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              匹配 {{ filteredPosts.length }} / {{ posts.length }}
            </div>
          </div>
          <div class="mt-3 flex flex-wrap items-center gap-2">
            <div class="relative" ref="tagMenuRef">
              <button
                type="button"
                class="meow-pill motion-press"
                :class="[
                  selectedTags.length > 0
                    ? (isNight ? 'border-meow-night-accent bg-meow-night-accent/15 text-meow-night-ink' : 'border-meow-accent bg-meow-accent/10 text-meow-ink')
                    : (isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : '')
                ]"
                @click.stop="toggleTagMenu"
              >
                标签筛选 {{ selectedTags.length > 0 ? `(${selectedTags.length})` : "" }}
              </button>
              <Transition name="tag-pop">
                <div
                  v-if="tagMenuOpen"
                  class="absolute left-0 top-[calc(100%+8px)] z-30 w-[min(78vw,460px)] rounded-2xl border p-3 shadow-[0_14px_30px_rgba(47,20,47,0.12)]"
                  :class="isNight ? 'border-meow-night-line bg-meow-night-card/95' : 'border-meow-line bg-white/95'"
                  @click.stop
                >
                  <div class="max-h-44 overflow-y-auto pr-1">
                    <div class="flex flex-wrap gap-2">
                      <button
                        type="button"
                        class="meow-pill motion-press"
                        :class="[
                          selectedTags.length === 0
                            ? (isNight ? 'border-meow-night-accent bg-meow-night-accent/15 text-meow-night-ink' : 'border-meow-accent bg-meow-accent/10 text-meow-ink')
                            : (isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : '')
                        ]"
                        @click="clearTagFilters"
                      >
                        全部标签
                      </button>
                      <button
                        v-for="tag in availableTags"
                        :key="`filter-${tag}`"
                        type="button"
                        class="meow-pill motion-press"
                        :class="[
                          selectedTags.includes(tag)
                            ? (isNight ? 'border-meow-night-accent bg-meow-night-accent/15 text-meow-night-ink' : 'border-meow-accent bg-meow-accent/10 text-meow-ink')
                            : (isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : '')
                        ]"
                        @click="toggleTagFilter(tag)"
                      >
                        #{{ tag }}
                      </button>
                    </div>
                  </div>
                  <div class="mt-3 flex items-center justify-between gap-2">
                    <span class="text-[11px]" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
                      {{ selectedTags.length > 0 ? `已选 ${selectedTags.length} 个标签` : "当前：全部标签" }}
                    </span>
                    <button
                      v-if="selectedTags.length > 0"
                      type="button"
                      class="meow-pill motion-press"
                      :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : ''"
                      @click="clearTagFilters"
                    >
                      清空标签
                    </button>
                  </div>
                </div>
              </Transition>
            </div>
            <span class="text-xs" :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'">
              {{ selectedTags.length > 0 ? selectedTags.join(" / ") : "全部标签" }}
            </span>
            <button
              v-if="searchQuery || selectedTags.length > 0"
              type="button"
              class="meow-pill motion-press"
              :class="isNight ? 'border-meow-night-line bg-meow-night-bg text-meow-night-soft' : ''"
              @click="clearFilters"
            >
              清除筛选
            </button>
          </div>
        </div>

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
          v-for="(post, index) in filteredPosts"
          :key="post.slug"
          class="meow-card motion-card rounded-3xl p-5 blog-reveal"
          :style="{ '--reveal-delay': `${index * 90}ms` }"
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
        <div
          v-if="!blogLoading && filteredPosts.length === 0"
          class="meow-card rounded-3xl p-5 text-sm"
          :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
        >
          没有符合条件的文章，试试更短的关键词或清除标签筛选。
        </div>
      </section>

      <section
        v-else-if="activePost"
        key="blog-detail"
        class="meow-card mt-8 rounded-3xl p-6"
        :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line' : ''"
      >
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
          <div
            class="blog-markdown text-sm leading-relaxed"
            :class="isNight ? 'text-meow-night-soft' : 'text-meow-soft'"
            v-html="activeContentHtml"
          ></div>
        </div>
      </section>

      <section
        v-else
        key="blog-not-found"
        class="meow-card mt-8 rounded-3xl p-6 text-sm"
        :class="isNight ? 'bg-meow-night-card/85 border-meow-night-line text-meow-night-soft' : 'text-meow-soft'"
      >
        没有找到这篇文章，返回列表看看其他内容吧。
      </section>
      </Transition>
    </main>
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

.blog-switch-enter-active,
.blog-switch-leave-active {
  transition: opacity 0.32s ease, transform 0.32s ease;
}

.blog-switch-enter-from,
.blog-switch-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

.top-action-fade-enter-active,
.top-action-fade-leave-active {
  transition: opacity 0.24s ease, transform 0.24s ease;
}

.top-action-fade-enter-from,
.top-action-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

.tag-pop-enter-active,
.tag-pop-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.tag-pop-enter-from,
.tag-pop-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

.blog-reveal {
  opacity: 0;
  transform: translateY(10px);
  animation: blogReveal 0.46s ease forwards;
  animation-delay: var(--reveal-delay, 0ms);
}

@keyframes blogReveal {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.blog-markdown > * {
  margin: 0.7em 0;
}

.blog-markdown h1,
.blog-markdown h2,
.blog-markdown h3,
.blog-markdown h4,
.blog-markdown h5,
.blog-markdown h6 {
  margin-top: 1em;
  margin-bottom: 0.45em;
  line-height: 1.3;
  color: inherit;
}

.blog-markdown h1 { font-size: 1.65rem; }
.blog-markdown h2 { font-size: 1.45rem; }
.blog-markdown h3 { font-size: 1.25rem; }

.blog-markdown ul,
.blog-markdown ol {
  margin: 0.75em 0;
  padding-left: 1.35em;
}

.blog-markdown li {
  margin: 0.3em 0;
}

.blog-markdown li input[type="checkbox"] {
  margin-right: 0.45em;
  transform: translateY(1px);
}

.blog-markdown blockquote {
  margin: 0.9em 0;
  padding: 0.55em 0.85em;
  border-left: 3px solid rgba(255, 122, 182, 0.55);
  background: rgba(255, 255, 255, 0.45);
  border-radius: 10px;
}

.meow-night .blog-markdown blockquote {
  border-left-color: rgba(136, 243, 255, 0.45);
  background: rgba(40, 33, 66, 0.72);
}

.blog-markdown pre {
  margin: 0.9em 0;
  padding: 0.8em 0.95em;
  border-radius: 12px;
  background: rgba(31, 25, 53, 0.92);
  color: #ece8ff;
  overflow-x: auto;
}

.blog-markdown code {
  padding: 0.1em 0.35em;
  border-radius: 7px;
  background: rgba(47, 20, 47, 0.08);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
}

.meow-night .blog-markdown code {
  background: rgba(136, 243, 255, 0.12);
}

.blog-markdown pre code {
  padding: 0;
  background: transparent;
}

.blog-markdown a {
  color: inherit;
  text-decoration: underline;
  text-underline-offset: 3px;
}

.blog-markdown del {
  opacity: 0.75;
}

.blog-markdown hr {
  border: 0;
  border-top: 1px solid rgba(207, 178, 203, 0.75);
  margin: 1em 0;
}

.meow-night .blog-markdown hr {
  border-top-color: rgba(96, 88, 139, 0.7);
}

.blog-markdown table {
  width: 100%;
  border-collapse: collapse;
  margin: 0.9em 0;
}

.blog-markdown th,
.blog-markdown td {
  border: 1px solid rgba(223, 203, 222, 0.9);
  padding: 0.45em 0.55em;
  text-align: left;
  vertical-align: top;
}

.blog-markdown th {
  background: rgba(255, 255, 255, 0.55);
}

.meow-night .blog-markdown th,
.meow-night .blog-markdown td {
  border-color: rgba(84, 77, 126, 0.9);
}

.meow-night .blog-markdown th {
  background: rgba(33, 28, 57, 0.88);
}

.blog-image {
  display: block;
  width: 100%;
  max-height: min(68vh, 560px);
  object-fit: contain;
  border: 1px solid rgba(233, 217, 234, 0.9);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.45);
  margin: 0.85em 0;
}
</style>
