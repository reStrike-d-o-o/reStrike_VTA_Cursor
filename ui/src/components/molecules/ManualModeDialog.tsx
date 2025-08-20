import React from 'react';
import Button from '../atoms/Button';
import { useI18n } from '../../i18n/index';

interface ManualModeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  isEnabled: boolean;
}

const ManualModeDialog: React.FC<ManualModeDialogProps> = ({
  isOpen,
  onClose,
  onConfirm,
  isEnabled
}) => {
  const { t } = useI18n();
  const handleConfirm = () => {
    onConfirm();
    onClose();
  };

  const handleCancel = () => {
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm border border-gray-600/30 rounded-lg p-6 w-96 max-w-[90vw] shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-4">
          {t('dock.manual_mode', 'Manual Mode')}
        </h2>
        <p className="text-gray-300 mb-6">
          {isEnabled ? t('dialog.manual.disable_confirm', 'Are you sure you want to disable manual mode?') : t('dialog.manual.enable_confirm', 'Are you sure you want to enable manual mode?')}
        </p>
        
        <div className="flex gap-3 justify-end">
          <Button
            type="button"
            variant="secondary"
            onClick={handleCancel}
            className="px-4 py-2"
          >
            {t('common.no', 'No')}
          </Button>
          <Button
            type="button"
            variant="primary"
            onClick={handleConfirm}
            className="px-4 py-2"
          >
            {t('common.yes', 'Yes')}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default ManualModeDialog; 