import { useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { saveEpub, exportTextPdfX, DEFAULT_PDF_SETTINGS } from '../tauri/commands';
import { useDocumentStore } from '../stores/documentStore';
import { notifyError } from '@/lib/utils/notifyError';

export function useFileExport() {
  const [isExporting, setIsExporting] = useState(false);
  const { currentContent, styles, metadata } = useDocumentStore();

  const handleExportEPUB = async () => {
    if (!currentContent) return;
    try {
      const cleanTitle = (metadata.title || 'Untitled')
        .replace(/[<>:"/\\|?*]/g, '_')
        .trim();
      const selected = await save({
        title: 'Export to EPUB',
        defaultPath: `${cleanTitle}.epub`,
        filters: [{ name: 'EPUB Ebook', extensions: ['epub'] }],
      });
      if (!selected) return;

      setIsExporting(true);
      const path = typeof selected === 'string' ? selected : (selected as any).path;
      if (!path) return;

      const bytes = await saveEpub(path, JSON.stringify(currentContent), styles, metadata, []);
      if (bytes && path.startsWith('content://')) await writeFile(path, bytes);
    } catch (error) {
      console.error('Failed to export EPUB:', error);
      notifyError('Failed to export EPUB', error);
      throw error;
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportPDF = async () => {
    if (!currentContent) return;
    try {
      const cleanTitle = (metadata.title || 'Untitled')
        .replace(/[<>:"/\\|?*]/g, '_')
        .trim();
      const selected = await save({
        title: 'Export to PDF/X',
        defaultPath: `${cleanTitle}.pdf`,
        filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
      });
      if (!selected) return;

      setIsExporting(true);
      const path = typeof selected === 'string' ? selected : (selected as any).path;
      if (!path) return;

      await exportTextPdfX(
        JSON.stringify(currentContent),
        styles,
        metadata,
        DEFAULT_PDF_SETTINGS,
        path,
      );
    } catch (error) {
      console.error('Failed to export PDF/X:', error);
      notifyError('Failed to export PDF/X', error);
      throw error;
    } finally {
      setIsExporting(false);
    }
  };

  return { handleExportEPUB, handleExportPDF, isExporting };
}
