import { getRaceTone, getSpiritVisualAssets } from '../../persona';
import { createBondProgress, createTalkChoices } from '../logic';
import type { SpiritProfilePanelProps } from '../types';

export function SpiritProfilePanel({
  activeDetail,
  styles,
  activeStyle,
  isSyncing,
  onSyncStyles,
  onSelectStyle,
}: SpiritProfilePanelProps) {
  const assets = activeDetail ? getSpiritVisualAssets(activeDetail) : null;
  const tone = activeDetail ? getRaceTone(activeDetail.race) : 'tone-neutral';
  const bond = createBondProgress(activeDetail);
  const choices = createTalkChoices(activeDetail);

  return (
    <aside className={`ever-profile ${tone}`}>
      {activeDetail ? (
        <>
          <section className="ever-profile-card">
            <p className="ever-kicker">BOND STATUS</p>
            <h2>{activeDetail.name}</h2>
            <span>{activeDetail.name_en}</span>
            <div className="ever-profile-bond">
              <strong>인연 {bond.level}Lv</strong>
              <span>{bond.current} / {bond.max}</span>
              <div className={`ever-bond-meter ${bond.meterClass}`}>
                <span />
              </div>
            </div>
            <div className="ever-profile-grid">
              <div><small>등급</small><strong>{activeDetail.grade}</strong></div>
              <div><small>종족</small><strong>{activeDetail.race}</strong></div>
              <div><small>클래스</small><strong>{activeDetail.class}</strong></div>
              <div><small>소속</small><strong>{activeDetail.profile?.union ?? '-'}</strong></div>
            </div>
          </section>

          <section className="ever-panel-section">
            <h3>대화 키워드</h3>
            <div className="ever-profile-choices">
              {choices.map((choice) => (
                <div key={choice.id} className={choice.rarity === 'rare' ? 'is-rare' : ''}>
                  <span>{choice.detail}</span>
                  <strong>{choice.label}</strong>
                  <small>{choice.reward}</small>
                </div>
              ))}
            </div>
          </section>

          <section className="ever-panel-section">
            <h3>개성 데이터</h3>
            <p>{activeDetail.personality?.description ?? '등록된 소개가 없습니다.'}</p>
            <div className="ever-tags">
              {activeDetail.profile?.like?.map((item) => <span key={`like-${item}`}>좋아함 {item}</span>)}
              {activeDetail.profile?.hobby?.map((item) => <span key={`hobby-${item}`}>취미 {item}</span>)}
            </div>
          </section>

          <section className="ever-panel-section">
            <h3>자산 연결</h3>
            <div className="ever-asset-status">
              <span>폴더</span>
              <strong>{assets?.assetFolder ?? '미연결'}</strong>
            </div>
            <div className="ever-asset-status">
              <span>배경</span>
              <strong>{assets?.background.split('/').slice(-1)[0]}</strong>
            </div>
          </section>

          <section className="ever-panel-section">
            <h3>말투 스타일</h3>
            {activeStyle && (
              <div className="ever-active-style">
                <strong>{activeStyle.name}</strong>
                <span>{activeStyle.tone} / {activeStyle.formality}</span>
              </div>
            )}
            {styles.length === 0 ? (
              <button className="ever-sync-button" type="button" disabled={isSyncing} onClick={onSyncStyles}>
                {isSyncing ? '동기화 중' : '서버 스타일 동기화'}
              </button>
            ) : (
              <div className="ever-style-list">
                {styles.map((style) => (
                  <button
                    key={style.id}
                    type="button"
                    className={style.is_active ? 'is-active' : ''}
                    onClick={() => onSelectStyle(style.id)}
                  >
                    {style.name}
                  </button>
                ))}
              </div>
            )}
          </section>
        </>
      ) : (
        <div className="ever-empty-panel">정령을 선택하면 TBL 기반 프로필과 원본 자산 연결 상태가 표시됩니다.</div>
      )}
    </aside>
  );
}
