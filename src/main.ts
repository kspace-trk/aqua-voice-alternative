import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { loadSettings, saveSettings, Settings } from './settings';
import './styles.css';

let settings: Settings;

// UI Elements
const apiKeyInput = document.getElementById('api-key') as HTMLInputElement;
const shortcutDisplay = document.getElementById('shortcut-display') as HTMLSpanElement;
const setShortcutBtn = document.getElementById('set-shortcut') as HTMLButtonElement;
const saveBtn = document.getElementById('save-settings') as HTMLButtonElement;
const statusIndicator = document.getElementById('status-indicator') as HTMLDivElement;
const statusText = document.getElementById('status-text') as HTMLSpanElement;

async function init() {
  settings = await loadSettings();
  console.log('Loaded settings:', settings);
  
  // Populate UI
  apiKeyInput.value = settings.apiKey;
  shortcutDisplay.textContent = settings.shortcut || 'Not set';
  
  // Set API key in Rust backend
  if (settings.apiKey) {
    await invoke('set_api_key', { apiKey: settings.apiKey });
  }
  
  // Register shortcut in Rust backend if exists
  if (settings.shortcut) {
    await registerShortcutInBackend(settings.shortcut);
  }
  
  updateStatus('idle');

  // Listen to status changes from Rust backend
  await listen<string>('status-changed', (event) => {
    const status = event.payload;
    console.log('Status changed:', status);

    if (status.startsWith('error:')) {
      const errorMessage = status.substring(6);
      updateStatus('error', errorMessage);
    } else {
      updateStatus(status as 'idle' | 'recording' | 'processing' | 'transcribing' | 'success' | 'error');
    }
  });
}

async function registerShortcutInBackend(shortcut: string) {
  try {
    await invoke('register_shortcut', { shortcutStr: shortcut });
    console.log(`Registered shortcut in backend: ${shortcut}`);
    updateStatus('success', 'Shortcut registered!');
    setTimeout(() => updateStatus('idle'), 2000);
  } catch (error) {
    console.error('Failed to register shortcut:', error);
    updateStatus('error', `Shortcut error: ${error}`);
  }
}

function updateStatus(status: 'idle' | 'recording' | 'processing' | 'transcribing' | 'success' | 'error', message?: string) {
  statusIndicator.className = 'status-indicator ' + status;
  
  const statusMessages: Record<string, string> = {
    idle: 'Ready (Recording handled by Rust backend)',
    recording: '🎙️ Recording...',
    processing: '⏳ Processing...',
    transcribing: '🔄 Transcribing...',
    success: message || '✅ Done',
    error: message || '❌ Error',
  };
  
  statusText.textContent = statusMessages[status];
}

// Event Listeners
setShortcutBtn.addEventListener('click', () => {
  shortcutDisplay.textContent = 'Press keys...';
  
  const handleKeyDown = async (e: KeyboardEvent) => {
    e.preventDefault();
    
    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push('CommandOrControl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    
    // Add the actual key (ignore modifier-only presses)
    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
      // Map special keys to their names
      const keyMap: Record<string, string> = {
        ' ': 'SPACE',
        'ArrowUp': 'UP',
        'ArrowDown': 'DOWN',
        'ArrowLeft': 'LEFT',
        'ArrowRight': 'RIGHT',
        'Enter': 'ENTER',
        'Escape': 'ESCAPE',
        'Tab': 'TAB',
        'Backspace': 'BACKSPACE',
        'Delete': 'DELETE',
        'Home': 'HOME',
        'End': 'END',
        'PageUp': 'PAGEUP',
        'PageDown': 'PAGEDOWN',
      };
      const keyName = keyMap[e.key] || e.key.toUpperCase();
      parts.push(keyName);
      
      const newShortcut = parts.join('+');
      shortcutDisplay.textContent = newShortcut;
      settings.shortcut = newShortcut;
      
      document.removeEventListener('keydown', handleKeyDown);
    }
  };
  
  document.addEventListener('keydown', handleKeyDown);
});

saveBtn.addEventListener('click', async () => {
  settings.apiKey = apiKeyInput.value.trim();
  
  try {
    await saveSettings(settings);
    
    // Update API key in Rust backend
    await invoke('set_api_key', { apiKey: settings.apiKey });
    
    // Re-register shortcut in Rust backend
    if (settings.shortcut) {
      await registerShortcutInBackend(settings.shortcut);
    }
    
    updateStatus('success', 'Settings saved!');
    setTimeout(() => updateStatus('idle'), 2000);
  } catch (error) {
    console.error('Failed to save settings:', error);
    updateStatus('error', `Save failed: ${error}`);
    setTimeout(() => updateStatus('idle'), 2000);
  }
});

// Initialize on load
init();
