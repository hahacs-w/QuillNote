<template>
  <div 
    class="settings-overlay" 
    @mousedown.self="isMouseDownOnOverlay = true" 
    @mouseup.self="handleOverlayMouseUp"
  >
    <div class="settings-modal">
      <div class="settings-header">
        <h2>Settings</h2>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>
      
      <div class="settings-content">
        <div class="settings-group">
          <label>Global Hotkey (唤起应用)</label>
          <input 
            v-model="config.hotkey" 
            @keydown.prevent="recordHotkey($event, 'hotkey')"
            placeholder="Click here and press keys..."
            class="settings-input hotkey-input"
          />
          <p class="settings-hint">Press any key combination (e.g. Cmd+Shift+Space) to record. Restart app to apply.</p>
        </div>

        <div class="settings-group">
          <label>Last Edit Hotkey (跳转最后编辑位置)</label>
          <input 
            v-model="config.last_edit_hotkey" 
            @keydown.prevent="recordHotkey($event, 'last_edit_hotkey')"
            placeholder="Click here and press keys..."
            class="settings-input hotkey-input"
          />
          <p class="settings-hint">Shortcut to jump to the last edited position.</p>
        </div>

        <div class="settings-group">
          <label>Storage Path</label>
          <div class="path-input-group">
            <input 
              v-model="config.storage_path" 
              placeholder="/Users/name/.draft_app_notes"
              class="settings-input"
            />
          </div>
          <div v-if="config.recent_storage_paths && config.recent_storage_paths.length > 0" class="recent-paths">
            <p class="recent-paths-title">Recently Used:</p>
            <div 
              v-for="path in config.recent_storage_paths" 
              :key="path" 
              class="recent-path-item"
            >
              <span 
                class="recent-path-text"
                @click="switchRecentPath(path)"
                title="Click to switch to this path"
              >
                {{ path }}
              </span>
              <button 
                class="remove-path-btn" 
                @click.stop="removeRecentPath(path)"
                title="Remove from list"
              >
                &times;
              </button>
            </div>
          </div>
          <p class="settings-hint">The directory where your SQLite database and HTML drafts are stored.</p>
        </div>

        <div class="settings-group">
          <label>Line Height: {{ config.line_height }}</label>
          <input 
            type="range" 
            v-model.number="config.line_height" 
            min="1.0" 
            max="2.5" 
            step="0.1"
            class="settings-slider"
          />
        </div>

        <div class="settings-group">
          <label>Paragraph Spacing: {{ config.paragraph_spacing }}em</label>
          <input 
            type="range" 
            v-model.number="config.paragraph_spacing" 
            min="0" 
            max="3" 
            step="0.1"
            class="settings-slider"
          />
        </div>
      </div>

      <div class="settings-footer">
        <button class="save-btn" @click="saveSettings">Save Changes</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits(['close']);

interface AppConfig {
  hotkey: string;
  storage_path: string;
  line_height: number;
  paragraph_spacing: number;
  last_edit_hotkey: string;
  recent_storage_paths: string[];
}

const config = ref<AppConfig>({
  hotkey: '',
  storage_path: '',
  line_height: 1.2,
  paragraph_spacing: 1.0,
  last_edit_hotkey: '',
  recent_storage_paths: []
});

const isMouseDownOnOverlay = ref(false);

const handleOverlayMouseUp = () => {
  if (isMouseDownOnOverlay.value) {
    emit('close');
  }
  isMouseDownOnOverlay.value = false;
};

const loadConfig = async () => {
  config.value = await invoke('get_config');
};

const recordHotkey = (e: KeyboardEvent, field: keyof AppConfig) => {
  // Ignore naked modifier keys
  if (['Control', 'Shift', 'Alt', 'Meta', 'Command'].includes(e.key)) {
    return;
  }

  const modifiers = [];
  if (e.metaKey || e.key === 'Meta') modifiers.push('Command');
  if (e.ctrlKey) modifiers.push('Control');
  if (e.altKey) modifiers.push('Alt');
  if (e.shiftKey) modifiers.push('Shift');

  let key = e.key.toUpperCase();
  if (key === ' ') key = 'SPACE';
  
  // Format: Modifier+Modifier+Key
  (config.value as any)[field] = [...modifiers, key].join('+');
};

const switchRecentPath = async (path: string) => {
  config.value.storage_path = path;
  await saveSettings();
};

const removeRecentPath = async (path: string) => {
  config.value.recent_storage_paths = config.value.recent_storage_paths.filter(p => p !== path);
  try {
    await invoke('save_config', { newConfig: config.value });
  } catch (e) {
    alert('Error removing path: ' + e);
    // Reload config if save failed to revert UI
    await loadConfig();
  }
};

const saveSettings = async () => {
  try {
    await invoke('save_config', { newConfig: config.value });
    alert('Settings saved successfully!');
    emit('close');
  } catch (e) {
    alert('Error saving settings: ' + e);
  }
};

onMounted(loadConfig);
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.settings-modal {
  background: var(--bg-editor);
  width: 450px;
  border-radius: 24px; /* Increased to match the much rounder app window */
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
}

.settings-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-secondary);
}

.settings-content {
  padding: 24px 20px;
}

.settings-group {
  margin-bottom: 20px;
}

.settings-group label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-primary);
}

.settings-input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-sidebar);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.settings-input:focus {
  border-color: #007aff;
}

.hotkey-input {
  text-align: center;
  font-weight: bold;
  letter-spacing: 1px;
  cursor: pointer;
  background: rgba(0, 122, 255, 0.05);
}

.settings-hint {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 6px;
}

.recent-paths {
  margin-top: 12px;
  background: rgba(128, 128, 128, 0.05);
  padding: 8px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.recent-paths-title {
  font-size: 10px;
  text-transform: uppercase;
  color: var(--text-secondary);
  margin: 0 0 6px 0;
  font-weight: 600;
}

.recent-path-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 6px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.recent-path-item:hover {
  background-color: rgba(0, 122, 255, 0.1);
}

.recent-path-text {
  font-size: 12px;
  color: #007aff;
  cursor: pointer;
  word-break: break-all;
  white-space: normal;
  line-height: 1.4;
  flex: 1;
}

.recent-path-text:hover {
  text-decoration: underline;
}

.remove-path-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 16px;
  cursor: pointer;
  padding: 0 4px;
  margin-left: 8px;
  opacity: 0.5;
  transition: opacity 0.2s, color 0.2s;
  line-height: 1;
}

.remove-path-btn:hover {
  opacity: 1;
  color: #ff3b30;
}

.settings-slider {
  width: 100%;
  cursor: pointer;
}

.settings-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.save-btn {
  background: #007aff;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.save-btn:hover {
  background: #0056b3;
}
</style>
