<template>
  <div class="app-layout" :class="{ 'focus-mode': isFocusMode }">
    <!-- Global Drag Bar (Absolute top) -->
    <div 
      class="window-drag-bar" 
      data-tauri-drag-region
      @mousedown="startWindowDrag"
    ></div>

    <!-- Left Pane: Sidebar (Folders & Tags) -->
    <aside class="sidebar">
      <div class="sidebar-search">
        <input 
          v-model="globalSearchQuery" 
          @input="performGlobalSearch"
          placeholder="Global Search..." 
          class="global-search-input"
        />
      </div>
      
      <div class="sidebar-section">
        <div class="section-title">Folders</div>
        <ul class="nav-list">
          <li 
            class="nav-item" 
            :class="{ 
              active: activeFolderId === null && activeTagId === null && !isGlobalSearching,
              'drag-over': dragOverFolderId === null
            }"
            @click="selectFolder(null)"
            data-folder-id="null"
          >
            <div class="folder-content">
              <span class="icon">📁</span> All Drafts
            </div>
          </li>
          <li
            v-for="folder in folders"
            :key="folder.id"
            class="nav-item"
            :class="{ 
              active: activeFolderId === folder.id && !isGlobalSearching,
              'drag-over': dragOverFolderId === folder.id 
            }"
            @click="selectFolder(folder.id)"
            @dblclick="startRenameFolder(folder)"
            :data-folder-id="folder.id"
          >
            <template v-if="renamingFolderId === folder.id">
              <input 
                v-model="renamingFolderName"
                @keyup.enter="finishRenameFolder"
                @blur="finishRenameFolder"
                class="new-folder-input"
                ref="renameFolderInputRef"
                @click.stop
                @mousedown.stop
                @dblclick.stop
              />
            </template>
            <template v-else>
              <div class="folder-content">
                <span class="icon">📁</span> {{ folder.name }}
              </div>
            </template>
          </li>          <!-- Inline New Folder Input -->
          <li v-if="isCreatingFolder" class="nav-item new-folder-input-item">
            <span class="icon">📁</span>
            <input 
              v-model="newFolderName" 
              @keydown.enter="submitNewFolder"
              @keydown.esc="cancelNewFolder"
              @blur="submitNewFolder"
              class="new-folder-input"
              ref="newFolderInputRef"
              placeholder="Folder name..."
            />
          </li>
        </ul>
        <button v-if="!isCreatingFolder" class="add-btn" @click="createNewFolder">+ New Folder</button>
      </div>

      <div class="sidebar-section">
        <div class="section-title">Tags</div>
        <ul class="nav-list">
          <li 
            v-for="tag in tags" 
            :key="tag.id"
            class="nav-item tag-item"
            :class="{ active: activeTagId === tag.id && !isGlobalSearching }"
            @click="selectTag(tag.id)"
          >
            <span class="icon">🏷️</span> {{ tag.name }}
          </li>
        </ul>
      </div>

      <div class="sidebar-footer">
        <button class="settings-btn" @click="showSettings = true">⚙️ Settings</button>
      </div>
    </aside>

    <!-- Middle Pane: Drafts List -->
    <section class="drafts-list">
      <div class="list-header" v-if="!isGlobalSearching">
        <button class="new-draft-btn" @click="createNewDraft">📝 Create Note</button>
      </div>
      <div class="list-header" v-else>
        <div class="search-status">Search Results</div>
      </div>
      <div class="draft-items">
        <template v-if="!isGlobalSearching">
          <div 
            v-for="draft in drafts" 
            :key="draft.id"
            class="draft-item"
            :class="{ active: activeDraft?.id === draft.id, dragging: draggingDraft?.id === draft.id }"
            @click="selectDraft(draft)"
            @dblclick="startRenameDraft(draft)"
            @mousedown="onDraftMouseDown($event, draft)"
          >
            <template v-if="renamingDraftId === draft.id">
              <input 
                v-model="renamingDraftTitle"
                @keyup.enter="finishRenameDraft"
                @blur="finishRenameDraft"
                class="draft-rename-input"
                ref="renameDraftInputRef"
                @click.stop
                @mousedown.stop
                @dblclick.stop
              />
            </template>
            <template v-else>
              <div class="draft-title">{{ draft.title || 'Untitled Draft' }}</div>
              <div class="draft-date">{{ formatDate(draft.updated_at) }}</div>
            </template>
          </div>
        </template>
        <template v-else>
          <div 
            v-for="result in searchResults" 
            :key="result.draft.id"
            class="draft-item"
            :class="{ active: activeDraft?.id === result.draft.id }"
            @click="selectDraft(result.draft)"
          >
            <div class="draft-title">{{ result.draft.title || 'Untitled Draft' }}</div>
            <div class="draft-excerpt">{{ result.excerpt }}</div>
            <div class="draft-date">{{ formatDate(result.draft.updated_at) }}</div>
          </div>
        </template>
        <div v-if="(!isGlobalSearching && drafts.length === 0) || (isGlobalSearching && searchResults.length === 0)" class="empty-state">
          No drafts found.
        </div>
      </div>
    </section>

    <!-- Right Pane: Editor -->
    <main class="editor-pane">
      <template v-if="activeDraft">
        <div class="editor-header" data-tauri-drag-region>
          <input 
            v-if="!activeSubDraft"
            v-model="activeDraft.title" 
            @input="updateDraftTitle"
            class="title-input" 
            placeholder="Draft Title"
          />
          <input 
            v-else
            v-model="activeSubDraft.title" 
            @input="updateDraftTitle"
            class="title-input sub-title-input" 
            placeholder="Sub-page Title"
          />
          
          <div class="header-right-spacer"></div>
          
          <!-- Export Dropdown -->
          <div class="export-dropdown" v-click-outside="() => isExportMenuOpen = false">
            <button class="export-btn" @click="isExportMenuOpen = !isExportMenuOpen" title="Export page">
              Export ▾
            </button>
            <Transition name="fade">
              <div v-if="isExportMenuOpen" class="export-menu">
                <div class="export-item" @click="exportFormat('txt')">Export as TXT</div>
                <div class="export-item" @click="exportFormat('word')">Export as Word (.docx)</div>
                <div class="export-item" @click="exportFormat('pdf')">Export as PDF</div>
                <div class="export-item" @click="exportFormat('html')">Export as HTML</div>
              </div>
            </Transition>
          </div>
          
          <button class="delete-btn" @click="deleteDraft" title="Delete current page">Trash</button>
        </div>
        
        <!-- Tags Bar -->
        <div class="editor-tags-bar">
          <span 
            v-for="tag in currentDraftTags" 
            :key="tag.id" 
            class="draft-tag"
          >
            {{ tag.name }}
            <span class="remove-tag" @click="removeTag(tag.id)">&times;</span>
          </span>
          <input 
            v-model="newTagInput" 
            @keydown.enter="addTag"
            placeholder="Add tag..." 
            class="add-tag-input"
          />
        </div>

        <div class="editor-container">
           <Editor 
             ref="editorRef"
             :draft-id="currentViewedDraft!.id" 
             :content-file="currentViewedDraft!.content_file" 
             :key="currentViewedDraft!.id"
             :line-height="appConfig?.line_height || 1.2"
             :paragraph-spacing="appConfig?.paragraph_spacing || 1.0"
             :jump-to-bookmark="pendingBookmarkDigit"
             :jump-to-pos="pendingJumpPos"
             :link-count="currentDraftLinks.length"
             :family-id="activeDraft?.id"
             @toggle-focus="toggleFocus"
             @bookmark-set="onBookmarkSet"
             @bookmark-not-found="onBookmarkNotFound"
             @clear-jump="pendingBookmarkDigit = null; pendingJumpPos = null"
             @open-links="openLinksModal"
             @jump-to-draft="handleJumpToDraft"
           />
           
           <div 
             class="sub-tabs-container draggable-sub-tabs"
             :style="{ transform: `translate(${tabsX}px, ${tabsY}px)` }"
             @mousedown="startTabsDrag"
           >
             <div class="drag-handle" title="Drag to move">⋮⋮</div>
             <button 
               class="sub-tab-btn" 
               :class="{ active: !activeSubDraft }"
               @click="switchToMainTab"
             >
               Main
             </button>
             <button 
               v-for="sub in subDrafts" 
               :key="sub.id"
               class="sub-tab-btn"
               :class="{ active: activeSubDraft?.id === sub.id }"
               @click="switchToSubTab(sub)"
               @dblclick="startRenameDraft(sub)"
               title="Double click to rename"
             >
               <template v-if="renamingDraftId === sub.id">
                 <input 
                   v-model="renamingDraftTitle"
                   @keyup.enter="finishRenameDraft"
                   @blur="finishRenameDraft"
                   class="sub-tab-rename-input"
                   ref="renameDraftInputRef"
                   @click.stop
                 />
               </template>
               <template v-else>
                 {{ sub.title }}
               </template>
             </button>
             <button class="add-sub-tab-btn" @click="createSubDraft" title="Add Sub-page">
               +
             </button>
           </div>
        </div>
      </template>
      <div v-else class="empty-editor">
        Select a draft or create a new one.
      </div>
    </main>

    <!-- Links Modal -->
    <div v-if="showLinksModal" class="modal-overlay" @click="showLinksModal = false">
      <div class="modal-content" @click.stop>
        <h3 class="modal-title">外部链接 (External Links)</h3>
        
        <div class="links-list">
          <div v-for="link in currentDraftLinks" :key="link.id" class="link-item">
            <div class="link-info" @click="openLink(link.url_or_path)" title="Click to open">
              <div class="link-alias" v-if="link.alias">{{ link.alias }}</div>
              <div class="link-url">{{ link.url_or_path }}</div>
              <div class="link-owner" style="font-size: 0.8em; color: #888;">归属: {{ getDraftTitle(link.draft_id) }}</div>
            </div>
            <div class="link-actions">
              <button class="edit-link-btn" @click="startEditLink(link)">Edit</button>
              <button class="delete-link-btn" @click="deleteLink(link.id)">Del</button>
            </div>
          </div>
          <div v-if="currentDraftLinks.length === 0" class="empty-links-state">暂无外链</div>
        </div>

        <div class="link-form">
          <select v-model="selectedOwnerDraftId" class="modal-input link-input">
            <option v-for="draft in ownerDrafts" :key="draft.id" :value="draft.id">
              {{ draft.title }} ({{ draft.parent_id ? '附属页' : '主页' }})
            </option>
          </select>
          <input 
            v-model="newLinkField1" 
            @input="handleLinkInput"
            placeholder="[别名](路径或URL) 或直接输入路径/URL" 
            class="modal-input link-input"
          />
          <input 
            v-model="newLinkField2" 
            placeholder="自定义别名 (选填)" 
            class="modal-input link-input"
          />
          <div class="modal-actions link-form-actions">
            <button class="cancel-btn" @click="showLinksModal = false">关闭</button>
            <div style="flex: 1;"></div>
            <button class="cancel-btn" v-if="editingLinkId" @click="cancelEditLink">取消</button>
            <button v-if="editingLinkId" class="primary-btn" @click="saveEditLink" :disabled="!newLinkField1.trim()">保存</button>
            <button v-else class="primary-btn" @click="addLink" :disabled="!newLinkField1.trim()">添加</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Settings Modal -->
    <Settings v-if="showSettings" @close="handleSettingsClose" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { save } from '@tauri-apps/plugin-dialog';
import html2pdf from 'html2pdf.js';
import { asBlob } from 'html-docx-js-typescript';
import Editor from './components/Editor.vue';
import Settings from './components/Settings.vue';
import type { DraftMeta, Folder, Tag, GlobalSearchResult, DraftLink } from './types';

const folders = ref<Folder[]>([]);
const tags = ref<Tag[]>([]);
const drafts = ref<DraftMeta[]>([]);

const activeFolderId = ref<string | null>(null);
const activeTagId = ref<string | null>(null);
const activeDraft = ref<DraftMeta | null>(null);

const subDrafts = ref<DraftMeta[]>([]);
const activeSubDraft = ref<DraftMeta | null>(null);
const lastActiveSubDraft = ref<DraftMeta | null>(null);
const currentViewedDraft = computed(() => activeSubDraft.value || activeDraft.value);

const newFolderInputRef = ref<HTMLInputElement | null>(null);

const editorRef = ref<InstanceType<typeof Editor> | null>(null);
const isExportMenuOpen = ref(false);
const pendingBookmarkDigit = ref<number | null>(null);
const pendingJumpPos = ref<number | null>(null);

const renameFolderInputRef = ref<HTMLInputElement | null>(null);
const renameDraftInputRef = ref<HTMLInputElement | null>(null);

const handleJumpToDraft = (payload: { draftId: string, pos: number }) => {
  if (!activeDraft.value) return;
  pendingJumpPos.value = payload.pos;

  if (payload.draftId === activeDraft.value.id) {
    switchToMainTab();
  } else {
    const sub = subDrafts.value.find(d => d.id === payload.draftId);
    if (sub) {
      switchToSubTab(sub);
    }
  }
};

// --- External Links State ---
const currentDraftLinks = ref<DraftLink[]>([]);
const showLinksModal = ref(false);
const newLinkField1 = ref('');
const newLinkField2 = ref('');
const editingLinkId = ref<string | null>(null);
const selectedOwnerDraftId = ref<string | null>(null);

const ownerDrafts = computed(() => {
  if (!activeDraft.value) return [];
  return [activeDraft.value, ...subDrafts.value];
});

const getDraftTitle = (id: string) => {
  const draft = ownerDrafts.value.find(d => d.id === id);
  return draft ? draft.title : '未知';
};

const loadDraftLinks = async (draftId: string) => {
  try {
    currentDraftLinks.value = await invoke('get_draft_links', { draftId });
  } catch (e) {
    console.error("Failed to load links:", e);
  }
};

const openLinksModal = () => {
  showLinksModal.value = true;
  newLinkField1.value = '';
  newLinkField2.value = '';
  editingLinkId.value = null;
  selectedOwnerDraftId.value = currentViewedDraft.value?.id || null;
};

const handleLinkInput = () => {
  const regex = /^\[(.*?)\]\((.*?)\)$/;
  const match = newLinkField1.value.match(regex);
  if (match) {
    newLinkField2.value = match[1];
    newLinkField1.value = match[2];
  }
};

const addLink = async () => {
  if (!newLinkField1.value.trim() || !selectedOwnerDraftId.value) return;
  await invoke('add_draft_link', {
    draftId: selectedOwnerDraftId.value,
    urlOrPath: newLinkField1.value.trim(),
    alias: newLinkField2.value.trim() || null
  });
  newLinkField1.value = '';
  newLinkField2.value = '';
  await loadDraftLinks(currentViewedDraft.value!.id);
};

const startEditLink = (link: DraftLink) => {
  editingLinkId.value = link.id;
  newLinkField1.value = link.url_or_path;
  newLinkField2.value = link.alias || '';
  selectedOwnerDraftId.value = link.draft_id;
};

const saveEditLink = async () => {
  if (!editingLinkId.value || !newLinkField1.value.trim() || !selectedOwnerDraftId.value || !currentViewedDraft.value) return;
  await invoke('update_draft_link', {
    id: editingLinkId.value,
    draftId: selectedOwnerDraftId.value,
    urlOrPath: newLinkField1.value.trim(),
    alias: newLinkField2.value.trim() || null
  });
  editingLinkId.value = null;
  newLinkField1.value = '';
  newLinkField2.value = '';
  await loadDraftLinks(currentViewedDraft.value!.id);
};

const cancelEditLink = () => {
  editingLinkId.value = null;
  newLinkField1.value = '';
  newLinkField2.value = '';
};

const deleteLink = async (id: string) => {
  if (!currentViewedDraft.value) return;
  await invoke('delete_draft_link', { id });
  await loadDraftLinks(currentViewedDraft.value.id);
};

const openLink = async (urlOrPath: string) => {
  try {
    await invoke('open_in_os', { path: urlOrPath });
  } catch (e) {
    console.error("Failed to open link:", e);
    alert(`无法打开链接: ${e}`);
  }
};
// ----------------------------

const onBookmarkSet = async (digit: number) => {
  if (!activeDraft.value) return;
  const family = [activeDraft.value, ...subDrafts.value];
  for (const draft of family) {
    if (draft.id === currentViewedDraft.value?.id) continue;
    try {
      const html = await invoke<string>('load_draft', { filename: draft.content_file });
      const regex = new RegExp(`<span[^>]*data-digit="${digit}"[^>]*>${digit}<\/span>`, 'g');
      if (regex.test(html)) {
        const newHtml = html.replace(regex, '');
        await invoke('save_draft', { content: newHtml, filename: draft.content_file });
      }
    } catch (e) {
      console.error('Error cleaning bookmark from other draft', e);
    }
  }
};

const onBookmarkNotFound = async (digit: number) => {
  if (!activeDraft.value) return;
  const family = [activeDraft.value, ...subDrafts.value];
  for (const draft of family) {
    if (draft.id === currentViewedDraft.value?.id) continue;
    try {
      const html = await invoke<string>('load_draft', { filename: draft.content_file });
      const regex = new RegExp(`data-digit="${digit}"`);
      if (regex.test(html)) {
        pendingBookmarkDigit.value = digit;
        if (draft.id === activeDraft.value.id) {
          switchToMainTab();
        } else {
          switchToSubTab(draft);
        }
        return;
      }
    } catch (e) {
      console.error('Error searching bookmark in other draft', e);
    }
  }
};

const exportFormat = async (format: 'txt' | 'word' | 'pdf' | 'html') => {
  isExportMenuOpen.value = false;
  console.log(`Starting export for format: ${format}`);
  
  if (!editorRef.value) {
    console.error('editorRef is null');
    return;
  }
  if (!currentViewedDraft.value) {
    console.error('currentViewedDraft is null');
    return;
  }

  let html = '';
  let text = '';
  try {
    html = editorRef.value.getEditorHTML();
    text = editorRef.value.getEditorText();
  } catch (err) {
    console.error('Failed to get editor content:', err);
    return;
  }

  const title = currentViewedDraft.value.title || 'Untitled';
  console.log(`Title: ${title}, HTML length: ${html.length}, Text length: ${text.length}`);
  
  if (!html && format !== 'txt') {
    console.warn('HTML is empty, skipping export');
    return;
  }
  if (!text && format === 'txt') {
    console.warn('Text is empty, skipping export');
    return;
  }

  try {
    if (format === 'txt') {
      console.log('Opening save dialog for TXT...');
      const filePath = await save({ defaultPath: `${title}.txt`, filters: [{ name: 'Text Document', extensions: ['txt'] }] });
      console.log('Selected path:', filePath);
      if (filePath) {
        await invoke('save_export_text', { path: filePath, content: text });
        console.log('TXT saved successfully');
      }
    } else if (format === 'html') {
      console.log('Opening save dialog for HTML...');
      const filePath = await save({ defaultPath: `${title}.html`, filters: [{ name: 'HTML Document', extensions: ['html'] }] });
      console.log('Selected path:', filePath);
      if (filePath) {
        const fullHtml = `<!DOCTYPE html>\n<html>\n<head>\n<meta charset="utf-8">\n<title>${title}</title>\n<style>body{font-family:sans-serif;line-height:1.6;padding:20px;max-width:800px;margin:0 auto;}</style>\n</head>\n<body>\n${html}\n</body>\n</html>`;
        await invoke('save_export_text', { path: filePath, content: fullHtml });
        console.log('HTML saved successfully');
      }
    } else if (format === 'word') {
      console.log('Opening save dialog for Word...');
      const filePath = await save({ defaultPath: `${title}.docx`, filters: [{ name: 'Word Document', extensions: ['docx'] }] });
      console.log('Selected path:', filePath);
      if (filePath) {
        const fullHtml = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${title}</title></head><body>${html}</body></html>`;
        const blob = (await asBlob(fullHtml)) as Blob;
        const arrayBuffer = await blob.arrayBuffer();
        await invoke('save_export_binary', { path: filePath, content: Array.from(new Uint8Array(arrayBuffer)) });
        console.log('Word saved successfully');
      }
    } else if (format === 'pdf') {
      console.log('Opening save dialog for PDF...');
      const filePath = await save({ defaultPath: `${title}.pdf`, filters: [{ name: 'PDF Document', extensions: ['pdf'] }] });
      console.log('Selected path:', filePath);
      if (filePath) {
        const element = document.createElement('div');
        element.innerHTML = html;
        element.style.padding = '20px';
        element.style.fontFamily = 'sans-serif';
        element.style.color = '#000';
        
        const opt = {
          margin:       10,
          filename:     'export.pdf',
          image:        { type: 'jpeg' as const, quality: 0.98 },
          html2canvas:  { scale: 2, useCORS: true },
          jsPDF:        { unit: 'mm', format: 'a4', orientation: 'portrait' as const }
        };
        
        console.log('Generating PDF blob...');
        const pdfWorker = html2pdf().set(opt).from(element);
        const pdfBlob = await pdfWorker.output('blob');
        const arrayBuffer = await pdfBlob.arrayBuffer();
        await invoke('save_export_binary', { path: filePath, content: Array.from(new Uint8Array(arrayBuffer)) });
        console.log('PDF saved successfully');
      }
    }
  } catch (error) {
    console.error(`Failed to export as ${format}:`, error);
    alert(`Failed to export document: ${error}`);
  }
};


// Tags state for active draft
const currentDraftTags = ref<Tag[]>([]);
const newTagInput = ref('');

// Global search state
const globalSearchQuery = ref('');
const isGlobalSearching = ref(false);
const searchResults = ref<GlobalSearchResult[]>([]);
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

const isFocusMode = ref(false);
const showSettings = ref(false);
const isCreatingFolder = ref(false);
const newFolderName = ref('');

interface AppConfig {
  hotkey: string;
  storage_path: string;
  line_height: number;
  paragraph_spacing: number;
}
const appConfig = ref<AppConfig | null>(null);

const loadAppConfig = async () => {
  appConfig.value = await invoke('get_config');
};

const handleSettingsClose = async () => {
  showSettings.value = false;
  await loadAppConfig();
  await loadFolders();
  await loadTags();
  await loadDrafts();
};

const toggleFocus = () => {
  console.log("Toggle focus triggered!");
  isFocusMode.value = !isFocusMode.value;
};

const startWindowDrag = async () => {
  await getCurrentWindow().startDragging();
};

const dragOverFolderId = ref<string | null | undefined>(undefined);
const draggingDraft = ref<DraftMeta | null>(null);

let dragGhost: HTMLElement | null = null;
let isDraggingDraft = false;
let dragStartX = 0;
let dragStartY = 0;
const DRAG_THRESHOLD = 5;

const onDraftMouseDown = (e: MouseEvent, draft: DraftMeta) => {
  // Don't start drag if it's on a rename input
  if ((e.target as HTMLElement).tagName === 'INPUT') return;
  // Only left button
  if (e.button !== 0) return;

  dragStartX = e.clientX;
  dragStartY = e.clientY;
  isDraggingDraft = false;
  const draftSnapshot = draft;

  const onMouseMove = (mv: MouseEvent) => {
    const dx = mv.clientX - dragStartX;
    const dy = mv.clientY - dragStartY;

    if (!isDraggingDraft && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      isDraggingDraft = true;
      draggingDraft.value = draftSnapshot;

      // Create ghost element
      dragGhost = document.createElement('div');
      dragGhost.className = 'drag-ghost';
      dragGhost.textContent = draftSnapshot.title || 'Untitled Draft';
      document.body.appendChild(dragGhost);
    }

    if (isDraggingDraft && dragGhost) {
      dragGhost.style.left = `${mv.clientX + 12}px`;
      dragGhost.style.top = `${mv.clientY + 12}px`; // Moved slightly lower to avoid any cursor overlap

      // Detect which folder is under cursor
      const el = document.elementFromPoint(mv.clientX, mv.clientY) as HTMLElement | null;

      const folderItem = el?.closest('[data-folder-id]') as HTMLElement | null;
      if (folderItem) {
        const fid = folderItem.getAttribute('data-folder-id');
        dragOverFolderId.value = fid === 'null' ? null : fid;
      } else {
        dragOverFolderId.value = undefined;
      }
    }
  };

  const onMouseUp = async (mu: MouseEvent) => {
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);

    if (dragGhost) {
      dragGhost.remove();
      dragGhost = null;
    }

    if (!isDraggingDraft) {
      draggingDraft.value = null;
      dragOverFolderId.value = undefined;
      return;
    }

    isDraggingDraft = false;

    // Detect drop target
    const el = document.elementFromPoint(mu.clientX, mu.clientY) as HTMLElement | null;
    const folderItem = el?.closest('[data-folder-id]') as HTMLElement | null;

    draggingDraft.value = null;
    dragOverFolderId.value = undefined;

    if (folderItem) {
      const fidAttr = folderItem.getAttribute('data-folder-id');
      const targetFolderId: string | null = fidAttr === 'null' ? null : fidAttr;

      if (draftSnapshot.folder_id === targetFolderId) return; // No change

      try {
        await invoke('update_draft_meta', {
          id: draftSnapshot.id,
          title: draftSnapshot.title,
          folderId: targetFolderId
        });
        if (isGlobalSearching.value) {
          performGlobalSearch();
        } else {
          await loadDrafts();
        }
      } catch (err) {
        console.error('Failed to move draft to folder:', err);
      }
    }
  };

  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
};

let titleUpdateTimeout: ReturnType<typeof setTimeout> | null = null;

const loadFolders = async () => {
  folders.value = await invoke('get_folders');
};

const loadTags = async () => {
  tags.value = await invoke('get_tags');
};

const loadDrafts = async () => {
  if (isGlobalSearching.value) return;
  drafts.value = await invoke('get_drafts', { 
    folderId: activeFolderId.value,
    tagId: activeTagId.value 
  });
  if (!activeDraft.value && drafts.value.length > 0) {
    selectDraft(drafts.value[0]);
  } else if (activeDraft.value) {
     const exists = drafts.value.find(d => d.id === activeDraft.value?.id);
     if (!exists) {
       activeDraft.value = null;
       currentDraftTags.value = [];
     }
  }
};

const loadDraftTags = async (draftId: string) => {
  currentDraftTags.value = await invoke('get_draft_tags', { draftId });
};

const selectFolder = async (folderId: string | null) => {
  isGlobalSearching.value = false;
  globalSearchQuery.value = '';
  activeFolderId.value = folderId;
  activeTagId.value = null;
  activeDraft.value = null;
  await loadDrafts();
};

const selectTag = async (tagId: string) => {
  isGlobalSearching.value = false;
  globalSearchQuery.value = '';
  activeTagId.value = tagId;
  activeFolderId.value = null;
  activeDraft.value = null;
  await loadDrafts();
};

const selectDraft = async (draft: DraftMeta) => {
  activeDraft.value = draft;
  activeSubDraft.value = null;
  lastActiveSubDraft.value = null;
  await loadSubDrafts(draft.id);
  await loadDraftTags(draft.id);
  await loadDraftLinks(draft.id);
};

const loadSubDrafts = async (parentId: string) => {
  subDrafts.value = await invoke('get_sub_drafts', { parentId });
};

const switchToMainTab = () => {
  if (activeSubDraft.value) {
    lastActiveSubDraft.value = activeSubDraft.value;
  }
  activeSubDraft.value = null;
  if (activeDraft.value) loadDraftLinks(activeDraft.value.id);
};

const switchToSubTab = (sub: DraftMeta) => {
  if (activeSubDraft.value?.id === sub.id) {
    lastActiveSubDraft.value = activeSubDraft.value;
    activeSubDraft.value = null; // Toggle back to main
    if (activeDraft.value) loadDraftLinks(activeDraft.value.id);
  } else {
    if (activeSubDraft.value) {
      lastActiveSubDraft.value = activeSubDraft.value;
    }
    activeSubDraft.value = sub;
    loadDraftLinks(sub.id);
  }
};

const createSubDraft = async () => {
  if (!activeDraft.value) return;
  const title = 'New Sub-page';
  const newSub: DraftMeta = await invoke('create_sub_draft', {
    title,
    parentId: activeDraft.value.id
  });
  await loadSubDrafts(activeDraft.value.id);
  activeSubDraft.value = newSub;
};

// --- Draggable Sub-Tabs ---
const tabsX = ref(0);
const tabsY = ref(0);

let isDraggingTabs = false;
let startMouseX = 0;
let startMouseY = 0;
let startTabsX = 0;
let startTabsY = 0;

const startTabsDrag = (e: MouseEvent) => {
  if ((e.target as HTMLElement).tagName === 'BUTTON') return;
  isDraggingTabs = true;
  startMouseX = e.clientX;
  startMouseY = e.clientY;
  startTabsX = tabsX.value;
  startTabsY = tabsY.value;
  
  const onMouseMove = (ev: MouseEvent) => {
    if (!isDraggingTabs) return;
    tabsX.value = startTabsX + (ev.clientX - startMouseX);
    tabsY.value = startTabsY + (ev.clientY - startMouseY);
  };
  
  const onMouseUp = () => {
    isDraggingTabs = false;
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);
  };
  
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
};

const renamingFolderId = ref<string | null>(null);
const renamingFolderName = ref('');
let isStartingRename = false;

const startRenameFolder = (folder: Folder) => {
  isStartingRename = true;
  renamingFolderId.value = folder.id;
  renamingFolderName.value = folder.name;
  setTimeout(() => {
    // When ref is in v-for, it's an array
    const el = renameFolderInputRef.value;
    const target = Array.isArray(el) ? el[0] : el;
    target?.focus();
    target?.select();
    setTimeout(() => { isStartingRename = false; }, 100);
  }, 50);
};

const finishRenameFolder = async () => {
  if (isStartingRename) return;
  if (renamingFolderId.value && renamingFolderName.value.trim()) {
    try {
      await invoke('rename_folder', { id: renamingFolderId.value, newName: renamingFolderName.value.trim() });
      await loadFolders();
      await loadDrafts();
    } catch (e) {
      console.error("Failed to rename folder:", e);
    }
  }
  renamingFolderId.value = null;
  renamingFolderName.value = '';
};

const renamingDraftId = ref<string | null>(null);
const renamingDraftTitle = ref('');

const startRenameDraft = (draft: DraftMeta) => {
  isStartingRename = true;
  renamingDraftId.value = draft.id;
  renamingDraftTitle.value = draft.title;
  setTimeout(() => {
    // When ref is in v-for, it's an array
    const el = renameDraftInputRef.value;
    const target = Array.isArray(el) ? el[0] : el;
    target?.focus();
    target?.select();
    setTimeout(() => { isStartingRename = false; }, 100);
  }, 50);
};

const finishRenameDraft = async () => {
  if (isStartingRename) return;
  if (renamingDraftId.value) {
    const newTitle = renamingDraftTitle.value.trim();
    if (newTitle) {
      await invoke('update_draft_meta', {
        id: renamingDraftId.value,
        title: newTitle,
        folderId: activeDraft.value?.id === renamingDraftId.value ? activeFolderId.value : null
      });
      
      if (isGlobalSearching.value) {
        performGlobalSearch();
      } else {
        await loadDrafts();
      }
      
      if (activeDraft.value && (renamingDraftId.value === activeDraft.value.id || subDrafts.value.some(s => s.id === renamingDraftId.value))) {
        if (renamingDraftId.value === activeDraft.value.id) {
          activeDraft.value.title = newTitle;
        } else {
          await loadSubDrafts(activeDraft.value.id);
        }
      }
    }
  }
  renamingDraftId.value = null;
  renamingDraftTitle.value = '';
};

const createNewFolder = () => {
  isCreatingFolder.value = true;
  newFolderName.value = '';
  nextTick(() => {
    newFolderInputRef.value?.focus();
  });
};

const submitNewFolder = async () => {
  if (!isCreatingFolder.value) return;
  const name = newFolderName.value.trim();
  if (name) {
    try {
      await invoke('create_folder', { name, parentId: null });
      await loadFolders();
    } catch (e) {
      console.error("Failed to create folder:", e);
    }
  }
  isCreatingFolder.value = false;
  newFolderName.value = '';
};

const cancelNewFolder = () => {
  isCreatingFolder.value = false;
  newFolderName.value = '';
};

const createNewDraft = async () => {
  const newDraft: DraftMeta = await invoke('create_draft', { 
    title: 'Untitled Draft', 
    folderId: activeFolderId.value 
  });
  await loadDrafts();
  selectDraft(newDraft);
};

const updateDraftTitle = () => {
  if (titleUpdateTimeout) clearTimeout(titleUpdateTimeout);
  titleUpdateTimeout = setTimeout(async () => {
    const target = currentViewedDraft.value;
    if (target) {
      await invoke('update_draft_meta', {
        id: target.id,
        title: target.title,
        folderId: target.folder_id
      });
      if (!activeSubDraft.value && !isGlobalSearching.value) {
        await loadDrafts();
      } else if (!activeSubDraft.value && isGlobalSearching.value) {
        const res = searchResults.value.find(r => r.draft.id === target.id);
        if (res) res.draft.title = target.title;
      } else if (activeSubDraft.value && activeDraft.value) {
        await loadSubDrafts(activeDraft.value.id);
      }
    }
  }, 500);
};

const deleteDraft = async () => {
  const target = currentViewedDraft.value;
  if (target && confirm('Are you sure you want to delete this page?')) {
    await invoke('delete_draft_meta', { id: target.id });
    await invoke('delete_draft_file', { filename: target.content_file });
    
    if (activeSubDraft.value) {
      activeSubDraft.value = null;
      if (activeDraft.value) await loadSubDrafts(activeDraft.value.id);
    } else {
      activeDraft.value = null;
      currentDraftTags.value = [];
      subDrafts.value = [];
      if (isGlobalSearching.value) {
        performGlobalSearch();
      } else {
        await loadDrafts();
      }
    }
  }
};

const addTag = async () => {
  const tag = newTagInput.value.trim();
  if (tag && activeDraft.value) {
    await invoke('add_tag_to_draft', { draftId: activeDraft.value.id, tagName: tag });
    newTagInput.value = '';
    await loadDraftTags(activeDraft.value.id);
    await loadTags();
  }
};

const removeTag = async (tagId: string) => {
  if (activeDraft.value) {
    await invoke('remove_tag_from_draft', { draftId: activeDraft.value.id, tagId: tagId });
    await loadDraftTags(activeDraft.value.id);
    await loadTags();
  }
};

const performGlobalSearch = () => {
  if (searchTimeout) clearTimeout(searchTimeout);
  if (!globalSearchQuery.value.trim()) {
    isGlobalSearching.value = false;
    loadDrafts();
    return;
  }
  
  isGlobalSearching.value = true;
  searchTimeout = setTimeout(async () => {
    try {
      searchResults.value = await invoke('global_search', { query: globalSearchQuery.value });
    } catch (e) {
      console.error("Search error:", e);
    }
  }, 300);
};

const formatDate = (isoString: string) => {
  const date = new Date(isoString);
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

let lastMetaPressTime = 0;

const handleGlobalKeydown = (e: KeyboardEvent) => {
  if (!activeDraft.value) return;

  if (e.key === 'Meta') {
    const now = Date.now();
    if (now - lastMetaPressTime < 400) {
      if (activeSubDraft.value) {
        switchToMainTab();
      } else if (lastActiveSubDraft.value) {
        const stillExists = subDrafts.value.find(d => d.id === lastActiveSubDraft.value?.id);
        if (stillExists) {
          switchToSubTab(stillExists);
        }
      }
      lastMetaPressTime = 0;
    } else {
      lastMetaPressTime = now;
    }
    return;
  }

  if (e.metaKey && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
    if (subDrafts.value.length > 0) {
      e.preventDefault();
      
      let currentIndex = -1;
      if (activeSubDraft.value) {
        currentIndex = subDrafts.value.findIndex(d => d.id === activeSubDraft.value?.id);
      }
      
      if (e.key === 'ArrowLeft') {
        if (currentIndex === -1) {
          switchToSubTab(subDrafts.value[subDrafts.value.length - 1]);
        } else if (currentIndex === 0) {
          switchToMainTab();
        } else {
          switchToSubTab(subDrafts.value[currentIndex - 1]);
        }
      } else if (e.key === 'ArrowRight') {
        if (currentIndex === -1) {
          switchToSubTab(subDrafts.value[0]);
        } else if (currentIndex === subDrafts.value.length - 1) {
          switchToMainTab();
        } else {
          switchToSubTab(subDrafts.value[currentIndex + 1]);
        }
      }
    }
  }
};

onMounted(async () => {
  await loadFolders();
  await loadTags();
  await loadDrafts();
  await loadAppConfig();
  window.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown);
});
</script>

<style>
:root {
  --bg-sidebar: #f5f5f7;
  --bg-list: #ffffff;
  --bg-editor: #ffffff;
  --border-color: rgba(0, 0, 0, 0.08);
  --text-primary: #1d1d1f;
  --text-secondary: #86868b;
  --active-bg: rgba(0, 122, 255, 0.08);
  --active-text: #007aff;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-sidebar: #1e1e1e;
    --bg-list: #252525;
    --bg-editor: #2c2c2c;
    --border-color: rgba(255, 255, 255, 0.08);
    --text-primary: #f5f5f7;
    --text-secondary: #a1a1a6;
  }
}

body, html, #app {
  margin: 0;
  padding: 0;
  height: 100vh;
  width: 100vw;
  background-color: transparent; /* Keep body transparent for rounded corners */
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-primary);
  user-select: none;
  -webkit-user-select: none;
}

/* Allow drag-and-drop on draft items */
.draft-item {
  -webkit-user-drag: element;
  user-select: auto;
  -webkit-user-select: auto;
}

.app-layout {
  display: grid;
  grid-template-columns: 200px 250px 1fr;
  height: 100%;
  width: 100%;
  border-radius: 12px; /* Apple Notes style rounded corners */
  border: 1px solid var(--border-color);
  overflow: hidden; /* Crucial: clips children to the border radius */
  background-color: var(--bg-editor);
  position: relative;
  /* Add a subtle shadow to make the white window pop against the desktop */
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15); 
}

.window-drag-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 15px;
  z-index: 999999;
  background: transparent;
  cursor: default;
}

/* Sidebar */
.sidebar {
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  padding: 15px 10px 16px 10px;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-search {
  margin-bottom: 16px;
}

.global-search-input {
  width: 100%;
  box-sizing: border-box;
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background-color: var(--bg-editor);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.global-search-input:focus {
  border-color: #007aff;
}

.sidebar-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-secondary);
  font-weight: 600;
  margin-bottom: 8px;
  padding-left: 8px;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  padding: 6px 8px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
  transition: background-color 0.1s, box-shadow 0.1s, transform 0.1s;
}

.nav-item:hover {
  background-color: rgba(128, 128, 128, 0.1);
}

.nav-item.active {
  background-color: var(--active-bg);
  color: var(--active-text);
  font-weight: 500;
}

.nav-item.drag-over {
  background-color: rgba(0, 122, 255, 0.25) !important;
  box-shadow: inset 0 0 0 2px #007aff !important;
  transform: scale(1.02);
}

.folder-content {
  pointer-events: none;
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.new-folder-input {
  background: transparent;
  border: none;
  border-bottom: 1px solid #007aff;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  width: 100%;
}

.add-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  padding: 8px;
  margin-top: 4px;
}

.add-btn:hover {
  color: var(--text-primary);
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--border-color);
}

.settings-btn {
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 12px;
  text-align: left;
  padding: 8px;
  cursor: pointer;
}

.settings-btn:hover {
  color: var(--text-primary);
}

/* Drafts List */
.drafts-list {
  background-color: var(--bg-list);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  padding-top: 15px;
}

.list-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.search-status {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
}

.new-draft-btn {
  width: 100%;
  padding: 6px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background-color: transparent;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 13px;
}

.new-draft-btn:hover {
  background-color: rgba(128, 128, 128, 0.05);
}

.draft-items {
  flex: 1;
  overflow-y: auto;
}

.draft-item {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  cursor: default;
  user-select: none;
  -webkit-user-select: none;
}

.draft-item:hover {
  background-color: rgba(128, 128, 128, 0.05);
  cursor: grab;
}

.draft-item.dragging {
  opacity: 0.4;
}

.draft-item.active {
  background-color: var(--active-bg);
}

/* Drag ghost element */
.drag-ghost {
  position: fixed;
  z-index: 99999;
  pointer-events: none;
  background-color: var(--bg-editor, #fff);
  border: 1px solid #007aff;
  border-radius: 8px;
  padding: 6px 12px;
  font-size: 13px;
  font-weight: 600;
  color: #007aff;
  box-shadow: 0 4px 16px rgba(0, 122, 255, 0.25);
  max-width: 200px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.92;
}

.draft-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.draft-excerpt {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.draft-date {
  font-size: 11px;
  color: var(--text-secondary);
}

.empty-state {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  color: var(--text-secondary);
}

/* Editor Pane */
.editor-pane {
  background-color: var(--bg-editor);
  display: flex;
  flex-direction: column;
  position: relative;
  min-height: 0;
  min-width: 0;
  padding-top: 15px;
}

.editor-header {
  display: flex;
  padding: 16px 24px 8px 24px;
  align-items: center;
  gap: 16px;
}

.title-input {
  flex: 1;
  font-size: 24px;
  font-weight: bold;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-primary);
  min-width: 0;
}

.header-right-spacer {
  flex: 1;
}

.sub-title-input {
  font-size: 20px;
  color: #007aff;
}

.sub-tabs-container {
  display: flex;
  gap: 6px;
  align-items: center;
  background-color: rgba(128, 128, 128, 0.05);
  padding: 4px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  overflow-x: auto;
  max-width: 100%;
}

.draggable-sub-tabs {
  position: absolute;
  bottom: 24px;
  right: 64px; /* Default right position avoiding scroll btn */
  z-index: 100;
  cursor: grab;
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  backdrop-filter: blur(10px);
  background-color: var(--bg-editor); /* Make background opaque for floating */
}

.draggable-sub-tabs:active {
  cursor: grabbing;
}

.drag-handle {
  padding: 0 4px;
  color: var(--text-secondary);
  opacity: 0.5;
  user-select: none;
  font-size: 14px;
  display: flex;
  align-items: center;
}

.sub-tab-btn {
  background: transparent;
  border: none;
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
  color: var(--text-secondary);
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.sub-tab-btn:hover {
  background-color: rgba(128, 128, 128, 0.1);
  color: var(--text-primary);
}

.sub-tab-btn.active {
  background-color: var(--bg-editor);
  color: var(--text-primary);
  box-shadow: 0 1px 4px rgba(0,0,0,0.1);
}

.add-sub-tab-btn {
  background: transparent;
  border: none;
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 16px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-sub-tab-btn:hover {
  background-color: rgba(128, 128, 128, 0.1);
  color: var(--text-primary);
}

.draft-rename-input {
  background: transparent;
  border: none;
  border-bottom: 1px solid #007aff;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  outline: none;
  width: 100%;
}

.sub-tab-rename-input {
  background: var(--bg-editor);
  border: 1px solid #007aff;
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  padding: 2px 4px;
  width: 80px;
}

.delete-btn {
  background: transparent;
  border: 1px solid rgba(255, 59, 48, 0.3);
  color: #ff3b30;
  border-radius: 6px;
  padding: 4px 12px;
  cursor: pointer;
  font-size: 12px;
  flex-shrink: 0;
}

.delete-btn:hover {
  background-color: rgba(255, 59, 48, 0.1);
}

.export-dropdown {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
}

.export-btn {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  border-radius: 6px;
  padding: 4px 12px;
  cursor: pointer;
  font-size: 12px;
}

.export-btn:hover {
  background-color: rgba(128, 128, 128, 0.05);
}

.export-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background-color: var(--bg-editor);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 150px;
  z-index: 100;
  padding: 4px 0;
}

.export-item {
  padding: 8px 16px;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background-color 0.2s;
  white-space: nowrap;
}

.export-item:hover {
  background-color: var(--active-bg);
  color: var(--active-text);
}

/* Tags Bar */
.editor-tags-bar {
  padding: 0 24px 12px 24px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.draft-tag {
  background-color: rgba(128, 128, 128, 0.1);
  border-radius: 12px;
  padding: 2px 8px;
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-primary);
}

.remove-tag {
  cursor: pointer;
  font-weight: bold;
  opacity: 0.5;
}

.remove-tag:hover {
  opacity: 1;
  color: #ff3b30;
}

.add-tag-input {
  border: none;
  outline: none;
  background: transparent;
  font-size: 11px;
  color: var(--text-primary);
  width: 80px;
}

.add-tag-input::placeholder {
  color: var(--text-secondary);
}

.editor-container {
  flex: 1;
  position: relative;
  min-height: 0;
}

.empty-editor {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 14px;
}

/* Focus Mode Overrides */
.app-layout.focus-mode {
  display: flex;
}

.app-layout.focus-mode .sidebar,
.app-layout.focus-mode .drafts-list,
.app-layout.focus-mode .editor-header,
.app-layout.focus-mode .editor-tags-bar {
  display: none !important;
}

.app-layout.focus-mode .editor-pane {
  flex: 1;
}

/* Links Modal Styling */
.modal-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: rgba(0, 0, 0, 0.4);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal-content {
  background-color: var(--bg-editor);
  border-radius: 12px;
  width: 480px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-color);
  padding: 20px;
}

.modal-title {
  margin: 0 0 16px 0;
  font-size: 16px;
  color: var(--text-primary);
}

.links-list {
  flex: 1;
  overflow-y: auto;
  margin-bottom: 20px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px;
}

.link-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
}

.link-item:hover {
  background-color: rgba(128, 128, 128, 0.05);
}

.link-info {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.link-alias {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-url {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-actions {
  display: flex;
  gap: 8px;
  margin-left: 12px;
}

.edit-link-btn, .delete-link-btn {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 11px;
  cursor: pointer;
  color: var(--text-secondary);
}

.edit-link-btn:hover {
  color: var(--text-primary);
  background-color: rgba(128, 128, 128, 0.1);
}

.delete-link-btn {
  border-color: rgba(255, 59, 48, 0.3);
  color: #ff3b30;
}

.delete-link-btn:hover {
  background-color: rgba(255, 59, 48, 0.1);
}

.empty-links-state {
  text-align: center;
  padding: 20px;
  color: var(--text-secondary);
  font-size: 12px;
}

.link-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
}

.link-input {
  border: 1px solid var(--border-color);
  background-color: transparent;
  border-radius: 6px;
  padding: 8px 12px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.link-input:focus {
  border-color: #007aff;
}

.link-form-actions {
  display: flex;
  gap: 8px;
}

.cancel-btn, .primary-btn {
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  border: none;
}

.cancel-btn {
  background-color: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.cancel-btn:hover {
  background-color: rgba(128, 128, 128, 0.05);
  color: var(--text-primary);
}

.primary-btn {
  background-color: #007aff;
  color: white;
}

.primary-btn:hover {
  background-color: #0056b3;
}

.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.links-btn {
  color: #007aff;
  border-color: rgba(0, 122, 255, 0.3);
}

.links-btn:hover {
  background-color: rgba(0, 122, 255, 0.1);
}
</style>

