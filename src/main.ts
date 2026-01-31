import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { invoke } from '@tauri-apps/api/core';
import { loadSettings, saveSettings, Settings } from './settings';
import { AudioRecorder, blobToBase64 } from './recorder';
import { transcribeAudio } from './transcriber';
import './styles.css';

let settings: Settings;
let recorder: AudioRecorder;
let isRecording = false;
let currentShortcut: string | null = null;

// UI Elements
const apiKeyInput = document.getElementById('api-key') as HTMLInputElement;
const shortcutDisplay = document.getElementById('shortcut-display') as HTMLSpanElement;
const setShortcutBtn = document.getElementById('set-shortcut') as HTMLButtonElement;
const saveBtn = document.getElementById('save-settings') as HTMLButtonElement;
const statusIndicator = document.getElementById('status-indicator') as HTMLDivElement;
const statusText = document.getElementById('status-text') as HTMLSpanElement;

async function init() {
  recorder = new AudioRecorder();
  settings = await loadSettings();
  console.log('Loaded settings:', settings);
  
  // Populate UI
  apiKeyInput.value = settings.apiKey;
  shortcutDisplay.textContent = settings.shortcut || 'Not set';
  
  // Register shortcut if exists
  if (settings.shortcut) {
    await registerShortcut(settings.shortcut);
  }
  
  updateStatus('idle');
}

async function registerShortcut(shortcut: string) {
  // Unregister previous shortcut
  if (currentShortcut) {
    try {
      await unregister(currentShortcut);
    } catch {
      // Ignore if already unregistered
    }
  }
  
  try {
    await register(shortcut, async (event) => {
      if (event.state === 'Pressed') {
        await startRecording();
      } else if (event.state === 'Released') {
        await stopRecordingAndTranscribe();
      }
    });
    currentShortcut = shortcut;
    console.log(`Registered shortcut: ${shortcut}`);
  } catch (error) {
    console.error('Failed to register shortcut:', error);
    throw error;
  }
}

async function startRecording() {
  if (isRecording) return;
  
  try {
    isRecording = true;
    updateStatus('recording');
    await recorder.start();
  } catch (error) {
    console.error('Failed to start recording:', error);
    isRecording = false;
    updateStatus('error', 'Failed to start recording');
  }
}

async function stopRecordingAndTranscribe() {
  if (!isRecording) return;
  
  try {
    updateStatus('processing');
    const audioBlob = await recorder.stop();
    isRecording = false;
    
    if (audioBlob.size === 0) {
      updateStatus('idle');
      return;
    }
    
    // Convert to base64
    const audioBase64 = await blobToBase64(audioBlob);
    
    // Transcribe with Gemini
    updateStatus('transcribing');
    const text = await transcribeAudio(settings.apiKey, audioBase64);
    
    if (text) {
      // Copy to clipboard
      await writeText(text);
      
      // Small delay before pasting
      await new Promise((resolve) => setTimeout(resolve, 100));
      
      // Paste using osascript
      await invoke('execute_paste');
      
      updateStatus('success', `Transcribed: ${text.substring(0, 50)}...`);
    } else {
      updateStatus('idle');
    }
    
    // Reset status after a short delay
    setTimeout(() => updateStatus('idle'), 3000);
  } catch (error) {
    console.error('Transcription failed:', error);
    isRecording = false;
    updateStatus('error', `Error: ${error}`);
    setTimeout(() => updateStatus('idle'), 3000);
  }
}

function updateStatus(status: 'idle' | 'recording' | 'processing' | 'transcribing' | 'success' | 'error', message?: string) {
  statusIndicator.className = 'status-indicator ' + status;
  
  const statusMessages: Record<string, string> = {
    idle: 'Ready',
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
      parts.push(e.key.toUpperCase());
      
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
    
    // Re-register shortcut with new settings
    if (settings.shortcut) {
      await registerShortcut(settings.shortcut);
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
