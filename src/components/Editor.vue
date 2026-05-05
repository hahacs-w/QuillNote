<template>
  <div 
    class="editor-wrapper" 
    :class="{ 'show-important-only': showImportantOnly }"
    :style="dynamicStyles"
  >
    
    <!-- Top Action Bar -->
    <div class="top-action-bar" data-tauri-drag-region>
      <button 
        class="filter-btn focus-toggle-btn"
        @click="emit('toggle-focus')"
        title="Toggle Focus Mode"
      >
        ⛶
      </button>
      <div class="center-buttons">
        <button 
          class="filter-btn" 
          :class="{ active: !showImportantOnly }"
          @click="toggleImportantOnly(false)"
        >
          全部文本
        </button>
        <button 
          class="filter-btn" 
          :class="{ active: showImportantOnly }"
          @click="toggleImportantOnly(true)"
        >
          只显示重要
        </button>
      </div>
      <div class="right-actions">
        <button 
          class="filter-btn links-btn"
          @click="emit('open-links')"
          title="管理外链"
        >
          外链{{ linkCount ? linkCount : '' }}
        </button>
        <button 
          class="icon-btn pin-btn" 
          :class="{ 'is-pinned': isPinned }"
          @click="togglePin"
          :title="isPinned ? '取消置顶' : '置顶'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" :fill="isPinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="17" x2="12" y2="22"></line>
            <path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
          </svg>
        </button>
        <label class="paste-format-label">
          <input type="checkbox" v-model="isRichText" />
          富文本
        </label>
      </div>
    </div>

    <!-- Tiptap Bubble Menu (PopClip style) -->
    <bubble-menu
      v-if="editor"
      :editor="editor"
      :tippy-options="{ duration: 100 }"
      class="bubble-menu"
    >
      <button 
        @click="editor.chain().focus().toggleHighlight().run()" 
        :class="{ 'is-active': editor.isActive('highlight') }"
        class="bubble-btn"
        title="标记重要"
      >
        ★ 重要
      </button>
      <button 
        @click="editor.chain().focus().toggleTrack().run()" 
        :class="{ 'is-active': editor.isActive('track') }"
        class="bubble-btn"
        title="标记轨迹"
      >
        📍 轨迹
      </button>
      <button 
        @click="editor.chain().focus().toggleBold().run()" 
        :class="{ 'is-active': editor.isActive('bold') }"
        class="bubble-btn"
      >
        B
      </button>
      <button 
        @click="editor.chain().focus().toggleItalic().run()" 
        :class="{ 'is-active': editor.isActive('italic') }"
        class="bubble-btn"
      >
        I
      </button>
    </bubble-menu>

    <!-- Local Search Overlay -->
    <Transition name="search-fade">
      <div v-if="isSearching" class="search-overlay">
        <div class="search-header">
          <input 
            v-model="searchQuery" 
            @input="performLocalSearch"
            @keydown.enter.prevent="nextMatch"
            @keydown.shift.enter.prevent="prevMatch"
            @keydown.esc.prevent="closeSearch"
            placeholder="搜索... (Enter 下一个, Esc 关闭)"
            class="search-input"
            ref="searchInputRef"
          />
          <span class="search-count">
            <template v-if="searchQuery && matchCount > 0">
              {{ currentMatchIndex + 1 }} / {{ matchCount }}
            </template>
            <template v-else-if="searchQuery && matchCount === 0">
              无结果
            </template>
          </span>
          <button @click="prevMatch" class="search-btn" title="上一个 (Shift+Enter)">↑</button>
          <button @click="nextMatch" class="search-btn" title="下一个 (Enter)">↓</button>
          <button @click="closeSearch" class="search-btn search-close-btn" title="关闭 (Esc)">✕</button>
        </div>
      </div>
    </Transition>

    <!-- Main Editor Area -->
    <div class="editor-scroll-area">
      <editor-content :editor="editor" class="editor-content" :data-time="formattedLastEditedTime" />
      
      <!-- Floating Scroll Buttons -->
      <Transition name="fade">
        <button 
          v-show="canScrollUp"
          class="scroll-btn scroll-to-top"
          @click="scrollToTop"
          title="回到顶部"
        >
          ↑
        </button>
      </Transition>
      <Transition name="fade">
        <button 
          v-show="canScrollDown"
          class="scroll-btn scroll-to-bottom"
          @click="scrollToBottom"
          title="到底部"
        >
          ↓
        </button>
      </Transition>

      <!-- Track Markers -->
      <div class="track-markers-container" v-if="trackMarkers.length > 0">
        <div 
          v-for="marker in trackMarkers" 
          :key="marker.id"
          class="track-marker"
          :style="{ top: marker.topPercentage + '%' }"
          @click="scrollToTrack(marker.element)"
          @dblclick="removeTrack(marker.element)"
          title="双击取消标记"
        >
          <span class="track-text" v-if="marker.text">{{ marker.text }}</span>
          <span class="track-arrow">◀</span>
        </div>
      </div>

      <!-- Bookmark Markers -->
      <div class="track-markers-container" v-if="bookmarkMarkers.length > 0">
        <div 
          v-for="marker in bookmarkMarkers" 
          :key="marker.id"
          class="bookmark-scroll-marker"
          :style="{ top: marker.topPercentage + '%' }"
          @click="scrollToTrack(marker.element)"
          :title="`书签 ${marker.digit}`"
        >
          {{ marker.digit }}
        </div>
      </div>
    </div>

  </div>
</template>

<script lang="ts">
// Shared state across all Editor component instances to remember scroll and cursor positions
interface EditorSavedState {
  scrollTop: number;
  selection?: { from: number; to: number };
}
const globalEditorState = new Map<string, EditorSavedState>();

// Shared state for jumping to the last edit position within the current draft family (main + subpages)
const globalFamilyLastEdit = new Map<string, { draftId: string; pos: number }>();
</script>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Editor, EditorContent } from '@tiptap/vue-3';
import { BubbleMenu } from '@tiptap/vue-3/menus';
import StarterKit from '@tiptap/starter-kit';
import Highlight from '@tiptap/extension-highlight';
import Image from '@tiptap/extension-image';
import Dropcursor from '@tiptap/extension-dropcursor';
import { Mark, Node, mergeAttributes, Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

const props = defineProps<{
  draftId: string;
  contentFile: string;
  lineHeight: number;
  paragraphSpacing: number;
  jumpToBookmark?: number | null;
  jumpToPos?: number | null;
  linkCount?: number;
  familyId?: string; // New prop to identify the draft family (main draft id)
}>();

const emit = defineEmits<{
  (e: 'toggle-focus'): void;
  (e: 'bookmark-set', digit: number): void;
  (e: 'bookmark-not-found', digit: number): void;
  (e: 'clear-jump'): void;
  (e: 'open-links'): void;
  (e: 'jump-to-draft', payload: { draftId: string, pos: number }): void;
}>();

const dynamicStyles = computed(() => ({
  '--editor-line-height': props.lineHeight,
  '--editor-paragraph-spacing': `${props.paragraphSpacing}em`
}));

// --- Custom Track Extension ---
const TrackMark = Mark.create({
  name: 'track',

  addOptions() {
    return {
      HTMLAttributes: {
        class: 'track-mark',
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'span.track-mark',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['span', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes), 0]
  },

  addCommands() {
    return {
      setTrack: () => ({ commands }) => {
        return commands.setMark(this.name)
      },
      toggleTrack: () => ({ commands }) => {
        return commands.toggleMark(this.name)
      },
      unsetTrack: () => ({ commands }) => {
        return commands.unsetMark(this.name)
      },
    }
  },
});

// --- Search Plugin (ProseMirror Decoration based) ---
const searchPluginKey = new PluginKey('search');

interface SearchState {
  matches: { start: number; end: number }[];
  currentIndex: number;
  query: string;
}

function buildSearchDecorations(doc: any, state: SearchState): DecorationSet {
  if (!state.query || state.matches.length === 0) return DecorationSet.empty;
  const decorations: Decoration[] = [];
  state.matches.forEach((m, i) => {
    const cls = i === state.currentIndex ? 'search-match-current' : 'search-match';
    decorations.push(Decoration.inline(m.start, m.end, { class: cls }));
  });
  return DecorationSet.create(doc, decorations);
}

const searchPlugin = new Plugin({
  key: searchPluginKey,
  state: {
    init(): SearchState { return { matches: [], currentIndex: 0, query: '' }; },
    apply(tr, old): SearchState {
      const meta = tr.getMeta(searchPluginKey);
      if (meta !== undefined) return meta as SearchState;
      // doc changed — recompute matches for same query
      if (tr.docChanged && old.query) {
        const matches: { start: number; end: number }[] = [];
        const q = old.query.toLowerCase();
        tr.doc.descendants((node, pos) => {
          if (node.isText && node.text) {
            const text = node.text.toLowerCase();
            let idx = text.indexOf(q);
            while (idx !== -1) {
              matches.push({ start: pos + idx, end: pos + idx + q.length });
              idx = text.indexOf(q, idx + 1);
            }
          }
        });
        const currentIndex = Math.min(old.currentIndex, Math.max(0, matches.length - 1));
        return { matches, currentIndex, query: old.query };
      }
      return old;
    },
  },
  props: {
    decorations(state) {
      const s: SearchState = searchPluginKey.getState(state);
      return buildSearchDecorations(state.doc, s);
    },
  },
});

// Tiptap wrapper extension for the search plugin
const SearchExtension = Extension.create({
  name: 'searchExtension',
  addProseMirrorPlugins() {
    return [searchPlugin];
  },
});

// --- Bookmark Node (Persistent) ---
const BookmarkNode = Node.create({
  name: 'bookmark',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      digit: {
        default: 0,
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span.bookmark-node',
        getAttrs: (dom) => {
          const digitAttr = (dom as HTMLElement).getAttribute('data-digit');
          return {
            digit: digitAttr ? parseInt(digitAttr, 10) : 0,
          };
        },
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(HTMLAttributes, {
        class: 'bookmark-node bookmark-badge',
        'data-digit': HTMLAttributes.digit,
        title: `书签 ${HTMLAttributes.digit} (快捷键: 跳转 Ctrl+${HTMLAttributes.digit}, 设置 Ctrl+Shift+${HTMLAttributes.digit})`
      }),
      String(HTMLAttributes.digit),
    ];
  },
});

// --- State ---
const isSearching = ref(false);
const searchQuery = ref('');
const searchInputRef = ref<HTMLInputElement | null>(null);

// Reactive display state (synced from plugin state after each dispatch)
const matchCount = ref(0);
const currentMatchIndex = ref(0);

const showImportantOnly = ref(false);
const savedScrollPosition = ref(0);

const toggleImportantOnly = (showImportant: boolean) => {
  if (showImportantOnly.value === showImportant) return;
  
  const container = document.querySelector('.editor-content');
  
  if (showImportant) {
    if (container) {
      savedScrollPosition.value = container.scrollTop;
    }
    showImportantOnly.value = true;
  } else {
    showImportantOnly.value = false;
    nextTick(() => {
      if (container) {
        container.scrollTop = savedScrollPosition.value;
      }
    });
  }
};

interface TrackMarker {
  id: string;
  topPercentage: number;
  element: HTMLElement;
  text: string;
}
const trackMarkers = ref<TrackMarker[]>([]);

interface BookmarkMarker {
  id: string;
  digit: number;
  topPercentage: number;
  element: HTMLElement;
}
const bookmarkMarkers = ref<BookmarkMarker[]>([]);

const appConfig = ref<any>({});
const lastEditPos = ref<number | null>(null);
const lastEditedTimestamp = ref<number | null>(null);

const formattedLastEditedTime = computed(() => {
  if (!lastEditedTimestamp.value) return '';
  const date = new Date(lastEditedTimestamp.value);
  
  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const hours = date.getHours();
  const minutes = date.getMinutes().toString().padStart(2, '0');
  
  const ampm = hours >= 12 ? '下午' : '上午';
  const displayHours = hours % 12 || 12;
  
  return `${year}年${month}月${day}日 ${ampm}${displayHours}:${minutes}`;
});

let saveTimeout: ReturnType<typeof setTimeout> | null = null;
const editor = ref<Editor>();
const isRichText = ref(true);

const isPinned = ref(false);
const togglePin = async () => {
  isPinned.value = !isPinned.value;
  try {
    await getCurrentWindow().setAlwaysOnTop(isPinned.value);
  } catch (error) {
    console.error("Failed to set window on top:", error);
    isPinned.value = !isPinned.value; // Revert on failure
  }
};

// --- Core Logic ---
const loadDraft = async () => {
  try {
    const html = await invoke<string>('load_draft', { filename: props.contentFile });
    const modTime = await invoke<number>('get_draft_modified_time', { filename: props.contentFile });
    
    if (modTime && modTime > 0) {
      lastEditedTimestamp.value = modTime;
    } else {
      lastEditedTimestamp.value = Date.now();
    }
    
    if (editor.value) {
      ignoreScrollEvents = true;
      editor.value.commands.setContent(html, { emitUpdate: false });
    }
    nextTick(() => {
      updateTracks();
      if (props.jumpToBookmark != null) {
        goToBookmark(props.jumpToBookmark);
        emit('clear-jump');
        ignoreScrollEvents = false;
      } else if (props.jumpToPos != null && editor.value) {
        const pos = props.jumpToPos;
        editor.value.chain()
          .focus()
          .setTextSelection(pos)
          .run();
          
        const container = document.querySelector('.editor-content');
        if (container) {
          try {
            const { view } = editor.value;
            const coords = view.coordsAtPos(pos);
            const containerRect = container.getBoundingClientRect();
            const absoluteTop = coords.top - containerRect.top + container.scrollTop;
            const centerPos = absoluteTop - container.clientHeight / 2;
            container.scrollTop = centerPos;
            checkScroll();
          } catch (err) {
            // Fallback
            container.scrollTop = container.scrollHeight;
          }
        }
        emit('clear-jump');
        ignoreScrollEvents = false;
      } else {
        const savedState = globalEditorState.get(props.draftId);
        if (savedState !== undefined) {
          // Use setTimeout to ensure TiPTap and browser layout calculations are complete
          setTimeout(() => {
            const container = document.querySelector('.editor-content');
            if (container) {
              container.scrollTop = savedState.scrollTop;
              // Trigger checkScroll manually to update visual states (scroll to top/bottom arrows)
              checkScroll();
            }
            if (editor.value && savedState.selection) {
              editor.value.commands.setTextSelection(savedState.selection);
            }
            ignoreScrollEvents = false;
          }, 100);
        } else {
          ignoreScrollEvents = false;
        }
      }
    });
  } catch (error) {
    console.error('Failed to load draft:', error);
    if (editor.value) {
      editor.value.commands.setContent('', { emitUpdate: false });
    }
  }
};

const saveDraft = async () => {
  if (!editor.value) return;
  try {
    const html = editor.value.getHTML();
    await invoke('save_draft', { 
      content: html, 
      filename: props.contentFile 
    });
    lastEditedTimestamp.value = Date.now();
    
    // Auto-update meta file with first line as title
    await invoke('update_draft_timestamp', { id: props.draftId });
  } catch (error) {
    console.error('Failed to save draft:', error);
  }
};

const onInput = () => {
  if (saveTimeout) clearTimeout(saveTimeout);
  saveTimeout = setTimeout(saveDraft, 500); // 500ms debounce
  if (isSearching.value) {
    performLocalSearch();
  }
  updateTracks();
};

// --- State for scrolling and tracking ---
const canScrollUp = ref(false);
const canScrollDown = ref(false);
const scrollTopValue = ref(0);

// Throttle checkScroll for better performance
let isScrolling = false;
let ignoreScrollEvents = false;

const checkScroll = () => {
  if (!isScrolling && !ignoreScrollEvents) {
    window.requestAnimationFrame(() => {
      const target = document.querySelector('.editor-content');
      if (target) {
        scrollTopValue.value = target.scrollTop;
        canScrollUp.value = target.scrollTop > 50;
        canScrollDown.value = Math.ceil(target.scrollTop + target.clientHeight) < target.scrollHeight - 50;
        const existing = globalEditorState.get(props.draftId) || { scrollTop: 0 };
        existing.scrollTop = target.scrollTop;
        globalEditorState.set(props.draftId, existing);
      }
      isScrolling = false;
    });
    isScrolling = true;
  }
};

const scrollToTop = () => {
  const container = document.querySelector('.editor-content');
  if (container) {
    container.scrollTo({ top: 0, behavior: 'smooth' });
  }
};

const scrollToBottom = () => {
  const container = document.querySelector('.editor-content');
  if (container) {
    container.scrollTo({ top: container.scrollHeight, behavior: 'smooth' });
  }
};

const updateTracks = () => {
  const container = document.querySelector('.editor-content');
  if (!container) return;
  
  checkScroll();

  const scrollHeight = container.scrollHeight;
  const containerRect = container.getBoundingClientRect();
  if (scrollHeight === 0) return;

  // Track Marks
  const marks = container.querySelectorAll('.track-mark');
  const markers: TrackMarker[] = [];

  marks.forEach((el, index) => {
    const element = el as HTMLElement;
    const rect = element.getBoundingClientRect();
    const absoluteTop = rect.top - containerRect.top + container.scrollTop;
    
    // Percentage relative to total scrollable height
    const topPercentage = (absoluteTop / scrollHeight) * 100;
    const text = element.innerText.trim().slice(0, 10);
    
    markers.push({
      id: `track-${index}`,
      topPercentage,
      element,
      text
    });
  });
  
  trackMarkers.value = markers;

  // Bookmark Marks
  const bookmarks = container.querySelectorAll('.bookmark-node');
  const bMarkers: BookmarkMarker[] = [];

  bookmarks.forEach((el, index) => {
    const element = el as HTMLElement;
    const digit = parseInt(element.getAttribute('data-digit') || '0', 10);
    const rect = element.getBoundingClientRect();
    const absoluteTop = rect.top - containerRect.top + container.scrollTop;
    
    const topPercentage = (absoluteTop / scrollHeight) * 100;
    
    bMarkers.push({
      id: `bookmark-${index}`,
      digit,
      topPercentage,
      element
    });
  });

  bookmarkMarkers.value = bMarkers;
};

const scrollToTrack = (element: HTMLElement) => {
  const container = document.querySelector('.editor-content');
  if (container) {
    const containerRect = container.getBoundingClientRect();
    const rect = element.getBoundingClientRect();
    const absoluteTop = rect.top - containerRect.top + container.scrollTop;
    container.scrollTo({ top: absoluteTop - 40, behavior: 'smooth' });
  }
};

const removeTrack = (element: HTMLElement) => {
  if (!editor.value) return;
  const pos = editor.value.view.posAtDOM(element, 0);
  if (pos !== null && pos >= 0) {
    editor.value.chain()
      .focus()
      .setTextSelection(pos)
      .extendMarkRange('track')
      .unsetMark('track')
      .run();
    nextTick(updateTracks);
  }
};

// --- Search Logic (Decoration-based) ---

/** Dispatch a new search state to the plugin and sync display refs */
const dispatchSearchState = (state: SearchState) => {
  if (!editor.value) return;
  const tr = editor.value.state.tr.setMeta(searchPluginKey, state);
  editor.value.view.dispatch(tr);
  matchCount.value = state.matches.length;
  currentMatchIndex.value = state.currentIndex;
};

/** Scroll the current decorated match element into view */
const scrollToCurrentMatch = () => {
  nextTick(() => {
    const current = document.querySelector('.search-match-current') as HTMLElement | null;
    if (current) {
      current.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }
  });
};

/** Compute matches from doc for a given query string */
const computeMatches = (query: string): { start: number; end: number }[] => {
  if (!editor.value || !query) return [];
  const q = query.toLowerCase();
  const result: { start: number; end: number }[] = [];
  editor.value.state.doc.descendants((node, pos) => {
    if (node.isText && node.text) {
      const text = node.text.toLowerCase();
      let idx = text.indexOf(q);
      while (idx !== -1) {
        result.push({ start: pos + idx, end: pos + idx + q.length });
        idx = text.indexOf(q, idx + 1);
      }
    }
  });
  return result;
};

const performLocalSearch = () => {
  const matches = computeMatches(searchQuery.value);
  dispatchSearchState({ matches, currentIndex: 0, query: searchQuery.value });
  if (matches.length > 0) scrollToCurrentMatch();
};

const nextMatch = () => {
  if (!editor.value) return;
  const s: SearchState = searchPluginKey.getState(editor.value.state);
  if (s.matches.length === 0) return;
  const next = (s.currentIndex + 1) % s.matches.length;
  dispatchSearchState({ ...s, currentIndex: next });
  scrollToCurrentMatch();
};

const prevMatch = () => {
  if (!editor.value) return;
  const s: SearchState = searchPluginKey.getState(editor.value.state);
  if (s.matches.length === 0) return;
  const prev = (s.currentIndex - 1 + s.matches.length) % s.matches.length;
  dispatchSearchState({ ...s, currentIndex: prev });
  scrollToCurrentMatch();
};

const clearSearch = () => {
  if (!editor.value) return;
  dispatchSearchState({ matches: [], currentIndex: 0, query: '' });
};

const closeSearch = () => {
  clearSearch();
  isSearching.value = false;
  searchQuery.value = '';
  editor.value?.commands.focus();
};

// --- Bookmark Logic ---
const setBookmark = (digit: number) => {
  if (!editor.value) return;
  const { state, view } = editor.value;
  let tr = state.tr;

  // First, find and remove any existing bookmark with the same digit
  state.doc.descendants((node, pos) => {
    if (node.type.name === 'bookmark' && node.attrs.digit === digit) {
      tr.delete(pos, pos + node.nodeSize);
    }
  });

  // Ensure the current selection position is correctly mapped after potential deletion
  const insertPos = tr.mapping.map(state.selection.from);
  const bookmarkNode = state.schema.nodes.bookmark.create({ digit });
  
  tr.insert(insertPos, bookmarkNode);
  view.dispatch(tr);
  
  emit('bookmark-set', digit);
};

const goToBookmark = (digit: number) => {
  if (!editor.value) return;
  const { state } = editor.value;
  let targetPos: number | null = null;
  
  state.doc.descendants((node, pos) => {
    if (node.type.name === 'bookmark' && node.attrs.digit === digit) {
      targetPos = pos;
    }
  });

  if (targetPos !== null) {
    editor.value.chain()
      .focus()
      .setTextSelection(targetPos)
      .run();

    // Small delay to ensure the DOM has rendered, especially after a page switch
    setTimeout(() => {
      const badge = document.querySelector(`.bookmark-node[data-digit="${digit}"]`) as HTMLElement;
      const container = document.querySelector('.editor-content');
      if (badge && container) {
        const containerRect = container.getBoundingClientRect();
        const badgeRect = badge.getBoundingClientRect();
        
        // Calculate the position relative to the scroll container's top
        const absoluteTop = badgeRect.top - containerRect.top + container.scrollTop;
        
        // Target: absoluteTop - (half of container height)
        // This puts the element precisely in the middle of the view
        const targetScrollTop = absoluteTop - (container.clientHeight / 2);
        
        container.scrollTo({ 
          top: targetScrollTop, 
          behavior: 'smooth' 
        });
      }
    }, 100);
  } else {
    emit('bookmark-not-found', digit);
  }
};

const matchHotkey = (e: KeyboardEvent, hotkeyStr: string) => {
  if (!hotkeyStr) return false;
  const parts = hotkeyStr.split('+');
  const key = parts.pop()?.toUpperCase();
  
  const hasCmd = parts.includes('Command');
  const hasCtrl = parts.includes('Control');
  const hasAlt = parts.includes('Alt');
  const hasShift = parts.includes('Shift');
  
  if (e.metaKey !== hasCmd) return false;
  if (e.ctrlKey !== hasCtrl) return false;
  if (e.altKey !== hasAlt) return false;
  if (e.shiftKey !== hasShift) return false;
  
  let eKey = e.key.toUpperCase();
  if (eKey === ' ') eKey = 'SPACE';
  
  return eKey === key;
};

const handleKeydown = async (e: KeyboardEvent) => {
  if (appConfig.value?.last_edit_hotkey && matchHotkey(e, appConfig.value.last_edit_hotkey)) {
    e.preventDefault();
    const familyKey = props.familyId || props.draftId;
    const lastEdit = globalFamilyLastEdit.get(familyKey);

    if (lastEdit) {
      if (lastEdit.draftId !== props.draftId) {
        // The last edit is in a different page within the family. Emit an event to switch.
        emit('jump-to-draft', { draftId: lastEdit.draftId, pos: lastEdit.pos });
        return;
      } else if (editor.value) {
        // The last edit is in this page.
        const pos = lastEdit.pos;
        editor.value.chain()
          .focus()
          .setTextSelection(pos)
          .run();
          
        nextTick(() => {
          const container = document.querySelector('.editor-content');
          if (container && editor.value) {
            try {
              const { view } = editor.value;
              const coords = view.coordsAtPos(pos);
              const containerRect = container.getBoundingClientRect();
              const absoluteTop = coords.top - containerRect.top + container.scrollTop;
              const centerPos = absoluteTop - container.clientHeight / 2;
              container.scrollTo({ top: centerPos, behavior: 'smooth' });
            } catch (err) {
              // Fallback
              container.scrollTo({ top: container.scrollHeight, behavior: 'smooth' });
            }
          }
        });
      }
    }
    return;
  }

  if (e.key === 'Escape') {
    if (isSearching.value) {
      closeSearch();
    } else {
      const win = getCurrentWindow();
      await win.hide();
    }
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
    e.preventDefault();
    isSearching.value = true;
    nextTick(() => {
      searchInputRef.value?.focus();
      const selection = window.getSelection()?.toString();
      if (selection) {
        searchQuery.value = selection;
        performLocalSearch();
      }
    });
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'g') {
    e.preventDefault();
    if (isSearching.value) {
      if (e.shiftKey) {
        prevMatch();
      } else {
        nextMatch();
      }
    }
  } else if ((e.ctrlKey || e.metaKey) && !e.altKey) {
    // Idea style bookmarks: Ctrl/Cmd + Shift + [0-9] to set, Ctrl/Cmd + [0-9] to go
    const match = e.code.match(/^(Digit|Numpad)(\d)$/);
    if (match) {
      const digit = parseInt(match[2], 10);
      e.preventDefault();
      if (e.shiftKey) {
        setBookmark(digit);
      } else {
        goToBookmark(digit);
      }
    }
  }
};

onMounted(async () => {
  try {
    appConfig.value = await invoke('get_config');
  } catch (err) {
    console.error('Failed to load config:', err);
  }

  editor.value = new Editor({
    content: '',
    extensions: [
      StarterKit,
      Highlight.configure({
        multicolor: false,
      }),
      Image.configure({
        inline: true,
        allowBase64: true,
      }),
      Dropcursor,
      TrackMark,
      SearchExtension,
      BookmarkNode,
    ],
    editorProps: {
      handleDOMEvents: {
        copy: (view, event) => {
          if (!isRichText.value) {
            const { from, to } = view.state.selection;
            const text = view.state.doc.textBetween(from, to, "\n");
            event.clipboardData?.setData('text/plain', text);
            event.preventDefault();
            return true;
          }
          return false;
        },
        cut: (view, event) => {
          if (!isRichText.value) {
            const { from, to } = view.state.selection;
            const text = view.state.doc.textBetween(from, to, "\n");
            event.clipboardData?.setData('text/plain', text);
            view.dispatch(view.state.tr.deleteSelection().scrollIntoView());
            event.preventDefault();
            return true;
          }
          return false;
        }
      },
      handlePaste(view, event) {
        if (!event.clipboardData) return false;
        const items = Array.from(event.clipboardData.items);
        let handled = false;
        for (const item of items) {
          if (item.type.startsWith('image/')) {
            const file = item.getAsFile();
            if (file) {
              const reader = new FileReader();
              reader.onload = (e) => {
                const src = e.target?.result as string;
                if (src) {
                  const node = view.state.schema.nodes.image.create({ src });
                  const tr = view.state.tr.replaceSelectionWith(node);
                  view.dispatch(tr);
                }
              };
              reader.readAsDataURL(file);
              handled = true;
            }
          }
        }
        if (handled) return true;

        if (!isRichText.value) {
          const text = event.clipboardData.getData('text/plain');
          if (text) {
            // Replace currently selected text with plain text
            view.dispatch(view.state.tr.replaceSelectionWith(
              view.state.schema.text(text)
            ));
            event.preventDefault();
            return true;
          }
        }

        return false;
      },
      handleDrop(view, event, _slice, moved) {
        if (!moved && event.dataTransfer && event.dataTransfer.files && event.dataTransfer.files.length > 0) {
          let handled = false;
          for (let i = 0; i < event.dataTransfer.files.length; i++) {
            const file = event.dataTransfer.files[i];
            if (file.type.startsWith('image/')) {
              const reader = new FileReader();
              reader.onload = (e) => {
                const src = e.target?.result as string;
                if (src) {
                  const coordinates = view.posAtCoords({ left: event.clientX, top: event.clientY });
                  const pos = coordinates ? coordinates.pos : view.state.selection.from;
                  const node = view.state.schema.nodes.image.create({ src });
                  const tr = view.state.tr.insert(pos, node);
                  view.dispatch(tr);
                }
              };
              reader.readAsDataURL(file);
              handled = true;
            }
          }
          if (handled) {
            event.preventDefault();
            return true;
          }
        }
        return false;
      }
    },
    onUpdate: ({ editor }) => {
      lastEditPos.value = editor.state.selection.from;
      const familyKey = props.familyId || props.draftId;
      globalFamilyLastEdit.set(familyKey, { draftId: props.draftId, pos: editor.state.selection.from });
      onInput();
    },
  });

  await loadDraft();
  window.addEventListener('keydown', handleKeydown);
  window.addEventListener('resize', updateTracks);
  
  const container = document.querySelector('.editor-content');
  if (container) {
    container.addEventListener('scroll', checkScroll);
  }
  
  const win = getCurrentWindow();
  win.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      if (isSearching.value && searchInputRef.value) {
        searchInputRef.value.focus();
      } else if (editor.value) {
        editor.value.commands.focus();
      }
      nextTick(updateTracks);
    }
  });
  
  if (editor.value) {
    editor.value.commands.focus();
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('resize', updateTracks);
  const container = document.querySelector('.editor-content');
  if (container) {
    container.removeEventListener('scroll', checkScroll);
  }
  if (editor.value) {
    const existing = globalEditorState.get(props.draftId) || { scrollTop: 0 };
    const { from, to } = editor.value.state.selection;
    existing.selection = { from, to };
    globalEditorState.set(props.draftId, existing);

    editor.value.destroy();
  }
});

defineExpose({
  getEditorHTML: () => {
    return editor.value?.getHTML() || '';
  },
  getEditorText: () => {
    return editor.value?.getText() || '';
  }
});
</script>

<style scoped>
.editor-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.top-action-bar {
  display: flex;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
  background-color: transparent;
}

.focus-toggle-btn {
  margin-right: auto;
  font-size: 16px;
  padding: 2px 8px;
}

.center-buttons {
  display: flex;
  gap: 10px;
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
}

.right-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-color, #888);
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  opacity: 0.8;
}

.icon-btn:hover {
  background-color: rgba(128, 128, 128, 0.1);
  opacity: 1;
}

.pin-btn.is-pinned {
  color: #007aff;
  background-color: rgba(0, 122, 255, 0.1);
  opacity: 1;
}

.pin-btn.is-pinned:hover {
  background-color: rgba(0, 122, 255, 0.15);
}

.paste-format-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-color, #666);
  cursor: pointer;
  user-select: none;
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", sans-serif;
  opacity: 0.8;
}

.paste-format-label:hover {
  opacity: 1;
}

.filter-btn {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 12px;
  padding: 4px 12px;
  font-size: 13px;
  color: var(--text-color, #333);
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", sans-serif;
}

.filter-btn:hover {
  background-color: rgba(128, 128, 128, 0.05);
}

.filter-btn.active {
  background-color: var(--text-color, #333);
  color: var(--bg-color, #fff);
  border-color: var(--text-color, #333);
}

.links-btn {
  color: #007aff;
  border-color: #007aff;
  background-color: rgba(0, 122, 255, 0.05);
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 14px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.links-btn:hover {
  background-color: #007aff;
  color: white;
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
  transform: translateY(-1px);
}

.links-btn:active {
  transform: translateY(0);
}

/* Bubble Menu (PopClip) */
.bubble-menu {
  display: flex;
  background-color: var(--bg-color, #fff);
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
  gap: 4px;
}

.bubble-btn {
  background: transparent;
  border: none;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text-color, #333);
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", sans-serif;
}

.bubble-btn:hover {
  background-color: rgba(128, 128, 128, 0.1);
}

.bubble-btn.is-active {
  background-color: rgba(255, 215, 0, 0.3);
  color: #d4a017;
  font-weight: bold;
}

/* Editor Area Layout */
.editor-scroll-area {
  flex: 1;
  position: relative;
  min-height: 0;
  display: flex;
  flex-direction: row;
}

.track-markers-container {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 16px;
  pointer-events: none;
  z-index: 50;
}

.track-marker {
  position: absolute;
  right: 4px;
  transform: translateY(-50%);
  color: #007aff;
  font-size: 10px;
  cursor: pointer;
  pointer-events: auto;
  user-select: none;
  transition: transform 0.1s ease, color 0.1s ease;
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.track-marker:hover {
  transform: translateY(-50%) scale(1.1);
  color: #0056b3;
}

.track-text {
  font-size: 9px;
  color: rgba(128, 128, 128, 0.85);
  background-color: var(--bg-color, #ffffff);
  padding: 2px 6px;
  border-radius: 4px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.1);
  opacity: 0.6;
  transition: opacity 0.2s ease, color 0.2s ease;
  border: 1px solid rgba(128, 128, 128, 0.15);
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", sans-serif;
}

.track-marker:hover .track-text {
  opacity: 1;
  color: #007aff;
  border-color: rgba(0, 122, 255, 0.3);
}

.track-arrow {
  transition: transform 0.2s;
}

.track-marker:hover .track-arrow {
  transform: scale(1.3);
}

.bookmark-scroll-marker {
  position: absolute;
  right: 4px;
  transform: translateY(-50%);
  color: white;
  background-color: #34c759;
  font-size: 9px;
  font-weight: bold;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  pointer-events: auto;
  user-select: none;
  transition: transform 0.1s ease, box-shadow 0.1s ease;
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}

.bookmark-scroll-marker:hover {
  transform: translateY(-50%) scale(1.3);
  box-shadow: 0 2px 5px rgba(0,0,0,0.4);
}

/* Tiptap Editor Content */
.editor-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 120px 40px 40px;
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  font-size: 13px;
  outline: none;
  user-select: text;
}

.editor-content::before {
  content: attr(data-time);
  display: block;
  text-align: center;
  font-size: 11px;
  color: var(--text-secondary, #999);
  padding: 24px 0 16px 0;
  font-weight: 500;
  user-select: none;
  opacity: 0.6;
  pointer-events: none;
}

.editor-content :deep(.tiptap) {
  outline: none;
  min-height: 100%;
  line-height: var(--editor-line-height, 1.2);
}

/* Exhaustively clear default browser styles */
.editor-content :deep(.tiptap p),
.editor-content :deep(.tiptap h1),
.editor-content :deep(.tiptap h2),
.editor-content :deep(.tiptap h3),
.editor-content :deep(.tiptap ul),
.editor-content :deep(.tiptap ol),
.editor-content :deep(.tiptap blockquote) {
  margin-top: 0;
  margin-bottom: var(--editor-paragraph-spacing, 1.0em);
  line-height: var(--editor-line-height, 1.2);
}

.editor-content :deep(.tiptap p:last-child) {
  margin-bottom: 0;
}

.editor-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  margin: 12px 0;
  display: inline-block;
  vertical-align: middle;
}

.editor-content :deep(img.ProseMirror-selectednode) {
  outline: 3px solid #007aff;
  box-shadow: 0 6px 20px rgba(0, 122, 255, 0.2);
}

.editor-content :deep(p.is-editor-empty:first-child::before) {
  content: "Type your thoughts here... (Select text to mark important or add tracks)";
  float: left;
  color: var(--placeholder-color, #999);
  pointer-events: none;
  height: 0;
}

/* Base Highlight */
.editor-content :deep(mark) {
  background-color: rgba(255, 215, 0, 0.3);
  color: inherit;
  border-radius: 2px;
  padding: 0 2px;
}

/* Track Mark Visuals */
.editor-content :deep(.track-mark) {
  border-bottom: 2px dashed #007aff;
  background-color: rgba(0, 122, 255, 0.1);
  padding: 0 2px;
  border-radius: 2px;
}

/* Show Important Only Logic */
.show-important-only .editor-content :deep(p),
.show-important-only .editor-content :deep(h1),
.show-important-only .editor-content :deep(h2),
.show-important-only .editor-content :deep(h3),
.show-important-only .editor-content :deep(ul),
.show-important-only .editor-content :deep(ol),
.show-important-only .editor-content :deep(blockquote) {
  display: none;
}

.show-important-only .editor-content :deep(p:has(mark)),
.show-important-only .editor-content :deep(h1:has(mark)),
.show-important-only .editor-content :deep(h2:has(mark)),
.show-important-only .editor-content :deep(h3:has(mark)),
.show-important-only .editor-content :deep(li:has(mark)),
.show-important-only .editor-content :deep(blockquote:has(mark)) {
  display: block;
  color: rgba(128, 128, 128, 0.3); 
}

.show-important-only .editor-content :deep(mark) {
  color: var(--text-color, #333);
  background-color: rgba(255, 215, 0, 0.5);
  font-weight: bold;
}


/* Search Fade Transition */
.search-fade-enter-active,
.search-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.search-fade-enter-from,
.search-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

/* Local Search Overlay Styling */
.search-overlay {
  position: absolute;
  top: 56px;
  right: 20px;
  width: 320px;
  background-color: var(--bg-color, #ffffff);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  z-index: 100;
  backdrop-filter: blur(16px);
}

.search-header {
  padding: 8px 10px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input {
  flex: 1;
  min-width: 0;
  padding: 5px 9px;
  font-size: 13px;
  border: 1px solid rgba(128, 128, 128, 0.35);
  border-radius: 6px;
  outline: none;
  background: transparent;
  color: var(--text-color, #333);
  font-family: "Sarasa Gothic TC", "更紗黑體 TC", sans-serif;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.search-input:focus {
  border-color: #007aff;
  box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.18);
}

.search-count {
  font-size: 11.5px;
  color: var(--placeholder-color, #999);
  white-space: nowrap;
  min-width: 44px;
  text-align: center;
}

.search-btn {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.28);
  border-radius: 5px;
  color: var(--text-color, #555);
  cursor: pointer;
  padding: 3px 7px;
  font-size: 12px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s;
  flex-shrink: 0;
}

.search-btn:hover {
  background-color: rgba(128, 128, 128, 0.12);
}

.search-close-btn {
  border-color: transparent;
  font-size: 11px;
  color: var(--placeholder-color, #aaa);
}

.search-close-btn:hover {
  color: var(--text-color, #333);
  background-color: rgba(128, 128, 128, 0.1);
}

/* Decoration-based search match highlights */
.editor-content :deep(.search-match) {
  background-color: rgba(255, 200, 0, 0.35);
  border-radius: 2px;
}

.editor-content :deep(.search-match-current) {
  background-color: rgba(255, 140, 0, 0.55);
  border-radius: 2px;
  outline: 1.5px solid rgba(255, 120, 0, 0.7);
}

/* Bookmark Badge */
.editor-content :deep(.bookmark-badge) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background-color: #34c759;
  color: white;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  height: 16px;
  min-width: 16px;
  padding: 0 4px;
  margin: 0 4px;
  vertical-align: middle;
  user-select: none;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

/* Floating Scroll Buttons */
.scroll-btn {
  position: absolute;
  right: 32px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: var(--bg-color, #ffffff);
  border: 1px solid rgba(128, 128, 128, 0.2);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  color: var(--text-color, #555);
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 40;
  transition: all 0.2s ease;
  backdrop-filter: blur(8px);
}

.scroll-btn:hover {
  background-color: rgba(128, 128, 128, 0.08);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.scroll-to-top {
  top: 16px;
}

.scroll-to-bottom {
  bottom: 24px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
</style>
