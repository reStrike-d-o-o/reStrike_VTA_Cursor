import React from 'react';

interface ModalProps {
  isOpen: boolean;
  title?: string;
  onClose: () => void;
  children: React.ReactNode;
  className?: string;
}

const Modal: React.FC<ModalProps> = ({ isOpen, title, onClose, children, className = '' }) => {
  if (!isOpen) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className={`relative theme-card p-0 shadow-lg w-[880px] max-w-[95vw] ${className}`} role="dialog" aria-modal>
        <div className="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-100">{title}</h3>
          <button className="text-gray-400 hover:text-white" onClick={onClose}>✕</button>
        </div>
        <div className="px-6 py-4 max-h-[70vh] overflow-auto">{children}</div>
      </div>
    </div>
  );
};

export default Modal;
