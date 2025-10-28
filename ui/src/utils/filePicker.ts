export const pickFilePath = async (extensions?: string[]): Promise<string | null> => {
  if (typeof window === 'undefined') {
    return null;
  }
  try {
    if ((window as any).__TAURI__ && (window as any).__TAURI__.core) {
      const dialog = await import('@tauri-apps/plugin-dialog');
      const selected = await dialog.open({
        multiple: false,
        filters: extensions && extensions.length ? [{ name: 'Files', extensions }] : undefined,
      });
      if (!selected) {
        return null;
      }
      if (Array.isArray(selected)) {
        return selected.length ? String(selected[0]) : null;
      }
      return typeof selected === 'string' ? selected : null;
    }
  } catch (error) {
    console.error('Unable to open native file dialog', error);
  }

  return await new Promise<string | null>((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    if (extensions?.length) {
      input.accept = extensions.map((ext) => `.${ext}`).join(',');
    }
    input.style.display = 'none';
    const cleanup = () => {
      if (input.parentNode) {
        input.parentNode.removeChild(input);
      }
    };
    input.addEventListener('change', () => {
      const file = input.files?.[0] ?? null;
      cleanup();
      if (!file) {
        resolve(null);
        return;
      }
      resolve(((file as any).path as string | undefined) || file.name || null);
    });
    input.addEventListener('cancel', () => {
      cleanup();
      resolve(null);
    });
    document.body.appendChild(input);
    input.click();
  });
};
