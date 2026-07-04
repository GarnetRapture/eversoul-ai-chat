import { useState } from 'react';
import { RotateCcw, X } from 'lucide-react';
import type { SettingsPanelProps } from '../types';
export function SettingsPanel({ open, settings, isResetting, resetSummary, resetError, onClose, onReset, }: SettingsPanelProps) {
    const [confirming, setConfirming] = useState(false);
    if (!open) {
        return null;
    }
    function handleResetClick() {
        if (!confirming) {
            setConfirming(true);
            return;
        }
        setConfirming(false);
        onReset();
    }
    function handleClose() {
        setConfirming(false);
        onClose();
    }
    return (<div className="ever-settings-overlay" role="dialog" aria-modal="true">
      <div className="ever-settings-modal">
        <header className="ever-settings-modal__header">
          <h2>설정</h2>
          <button type="button" aria-label="설정 닫기" onClick={handleClose}>
            <X aria-hidden="true" size={20}/>
          </button>
        </header>

        <section className="ever-panel-section">
          <h3>현재 설정값 (settings.ini)</h3>
          <div className="ever-profile-grid">
            <div>
              <small>기본 정령</small>
              <strong>{settings?.default_persona_id ?? '미지정'}</strong>
            </div>
            <div>
              <small>활성 스타일</small>
              <strong>{settings?.active_style_id ?? '미지정'}</strong>
            </div>
          </div>
        </section>

        <section className="ever-panel-section ever-settings-danger">
          <h3>데이터 초기화</h3>
          <p>
            대화 기록, 정령/스타일/지식팩 데이터, 정령별 누적 기억(개조학습 데이터)과
            설정값(settings.ini)을 모두 삭제해 앱을 초기 상태로 되돌립니다. 정령 프로필은
            다음 로드 시 번들 아카이브로부터 자동으로 다시 채워지지만, 대화 기록과 정령이
            그동안 누적한 기억은 복구할 수 없습니다.
          </p>

          {resetSummary && (<div className="ever-settings-result">
              <strong>초기화 완료</strong>
              <span>대화방 {resetSummary.cleared_chat_rooms}개</span>
              <span>메시지 {resetSummary.cleared_chat_messages}개</span>
              <span>정령 프로필 {resetSummary.cleared_personas}개</span>
              <span>스타일 {resetSummary.cleared_styles}개</span>
              <span>지식 청크 {resetSummary.cleared_knowledge_chunks}개</span>
              <span>누적 기억 {resetSummary.cleared_persona_memories}개</span>
            </div>)}

          {resetError && (<div className="ever-roster__error">
              <strong>초기화 실패</strong>
              <span>{resetError}</span>
            </div>)}

          <button type="button" className={`ever-settings-reset-button ${confirming ? 'is-confirming' : ''}`} disabled={isResetting} onClick={handleResetClick}>
            <RotateCcw aria-hidden="true" size={16}/>
            {isResetting ? '초기화 중...' : confirming ? '정말 초기화하시겠습니까? (다시 클릭 시 실행)' : '모든 데이터 초기화'}
          </button>
        </section>
      </div>
    </div>);
}
