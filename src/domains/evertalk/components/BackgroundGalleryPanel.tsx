import { useMemo, useState } from 'react';
import { ChevronLeft, ChevronRight, X, ZoomIn } from 'lucide-react';
import { ASSET_ROOT } from '../../persona';
import { BACKGROUND_ASSET_FILES } from '../backgroundAssets';
import type { BackgroundGalleryPanelProps } from '../types';

const PAGE_SIZE = 60;

export function BackgroundGalleryPanel({ open, onClose }: BackgroundGalleryPanelProps) {
  const [zoomedFile, setZoomedFile] = useState<string | null>(null);
  const [page, setPage] = useState(0);

  const totalPages = Math.max(1, Math.ceil(BACKGROUND_ASSET_FILES.length / PAGE_SIZE));
  const pageFiles = useMemo(
    () => BACKGROUND_ASSET_FILES.slice(page * PAGE_SIZE, page * PAGE_SIZE + PAGE_SIZE),
    [page]
  );

  if (!open) {
    return null;
  }

  function handleClose() {
    setZoomedFile(null);
    setPage(0);
    onClose();
  }

  function goToPage(next: number) {
    setPage(Math.min(Math.max(next, 0), totalPages - 1));
  }

  return (
    <div className="ever-settings-overlay" role="dialog" aria-modal="true">
      <div className="ever-background-gallery-modal">
        <header className="ever-settings-modal__header">
          <h2>배경 갤러리 ({BACKGROUND_ASSET_FILES.length}개)</h2>
          <button type="button" aria-label="배경 갤러리 닫기" onClick={handleClose}>
            <X aria-hidden="true" size={20} />
          </button>
        </header>
        <div className="ever-background-gallery-grid">
          {pageFiles.map((file) => (
            <button
              key={file}
              type="button"
              className="ever-gallery-tile ever-gallery-tile--button"
              aria-label={`${file} 확대 보기`}
              onClick={() => setZoomedFile(file)}
            >
              <img src={`${ASSET_ROOT}/backgrounds/talk/${file}`} alt={file} loading="lazy" />
              <span className="ever-gallery-tile__zoom-hint" aria-hidden="true">
                <ZoomIn size={18} />
              </span>
            </button>
          ))}
        </div>
        <div className="ever-background-gallery-pager">
          <button
            type="button"
            aria-label="이전 페이지"
            disabled={page === 0}
            onClick={() => goToPage(page - 1)}
          >
            <ChevronLeft aria-hidden="true" size={18} />
          </button>
          <span>{page + 1} / {totalPages} 페이지</span>
          <button
            type="button"
            aria-label="다음 페이지"
            disabled={page >= totalPages - 1}
            onClick={() => goToPage(page + 1)}
          >
            <ChevronRight aria-hidden="true" size={18} />
          </button>
        </div>
      </div>

      {zoomedFile && (
        <div
          className="ever-background-zoom-overlay"
          role="dialog"
          aria-modal="true"
          onClick={() => setZoomedFile(null)}
        >
          <button
            type="button"
            className="ever-background-zoom-close"
            aria-label="확대 보기 닫기"
            onClick={() => setZoomedFile(null)}
          >
            <X aria-hidden="true" size={24} />
          </button>
          <img
            className="ever-background-zoom-image"
            src={`${ASSET_ROOT}/backgrounds/talk/${zoomedFile}`}
            alt={zoomedFile}
            onClick={(event) => event.stopPropagation()}
          />
          <span className="ever-background-zoom-caption">{zoomedFile}</span>
        </div>
      )}
    </div>
  );
}
