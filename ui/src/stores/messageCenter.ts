import { create } from 'zustand';

export type MessageSeverity = 'info' | 'success' | 'warning' | 'error';

type ModalKind = 'message' | 'confirm';

export interface ModalItem {
  id: string;
  kind: ModalKind;
  severity: MessageSeverity;
  title: string;
  body?: string;
  confirmText?: string;
  cancelText?: string;
  /** Internal: resolver for confirm modal */
  _resolve?: (ok: boolean) => void;
}

interface MessageCenterState {
  current: ModalItem | null;
  queue: ModalItem[];

  showMessage: (args: { severity?: MessageSeverity; title: string; body?: string }) => void;
  showError: (title: string, body?: string) => void;
  showWarning: (title: string, body?: string) => void;
  showInfo: (title: string, body?: string) => void;
  showSuccess: (title: string, body?: string) => void;
  confirm: (args: { title: string; body?: string; confirmText?: string; cancelText?: string; severity?: MessageSeverity }) => Promise<boolean>;
  close: (ok?: boolean) => void;
  /** Internal: advance queue when current is empty */
  _ensureCurrent: () => void;
}

const makeId = () => Math.random().toString(36).slice(2);

export const useMessageCenter = create<MessageCenterState>((set, get) => ({
  current: null,
  queue: [],

  showMessage: ({ severity = 'info', title, body }) => {
    const item: ModalItem = {
      id: makeId(),
      kind: 'message',
      severity,
      title,
      body,
    };
    set((state) => ({ queue: [...state.queue, item] }));
    get()._ensureCurrent();
  },

  showError: (title, body) => get().showMessage({ severity: 'error', title, body }),
  showWarning: (title, body) => get().showMessage({ severity: 'warning', title, body }),
  showInfo: (title, body) => get().showMessage({ severity: 'info', title, body }),
  showSuccess: (title, body) => get().showMessage({ severity: 'success', title, body }),

  confirm: ({ title, body, confirmText = 'Confirm', cancelText = 'Cancel', severity = 'warning' }) => {
    return new Promise<boolean>((resolve) => {
      const item: ModalItem = {
        id: makeId(),
        kind: 'confirm',
        severity,
        title,
        body,
        confirmText,
        cancelText,
        _resolve: resolve,
      };
      set((state) => ({ queue: [...state.queue, item] }));
      get()._ensureCurrent();
    });
  },

  close: (ok = true) => {
    const current = get().current;
    if (current?.kind === 'confirm' && current._resolve) {
      current._resolve(!!ok);
    }
    set((state) => ({ current: null }));
    get()._ensureCurrent();
  },

  _ensureCurrent: () => {
    const { current, queue } = get();
    if (!current && queue.length > 0) {
      const [next, ...rest] = queue;
      set({ current: next, queue: rest });
    }
  },
}));


