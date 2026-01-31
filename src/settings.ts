export interface Settings {
  apiKey: string;
  shortcut: string;
}

export const DEFAULT_SETTINGS: Settings = {
  apiKey: '',
  shortcut: 'CommandOrControl+Shift+Space',
};

export async function loadSettings(): Promise<Settings> {
  try {
    const { readTextFile, BaseDirectory } = await import('@tauri-apps/plugin-fs');
    const content = await readTextFile('settings.json', { baseDir: BaseDirectory.AppData });
    return { ...DEFAULT_SETTINGS, ...JSON.parse(content) };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export async function saveSettings(settings: Settings): Promise<void> {
  const { writeTextFile, mkdir, BaseDirectory } = await import('@tauri-apps/plugin-fs');
  
  try {
    await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true });
  } catch {
    // Directory may already exist
  }
  
  await writeTextFile('settings.json', JSON.stringify(settings, null, 2), {
    baseDir: BaseDirectory.AppData,
  });
}
