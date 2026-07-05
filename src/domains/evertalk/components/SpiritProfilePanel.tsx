import { BrainCircuit, Images, PanelRightClose, PanelRightOpen, Settings } from 'lucide-react';
import { getRaceTone, getSpiritVisualAssets } from '../../persona';
import { createTalkChoices } from '../logic';
import type { SpiritProfilePanelProps } from '../types';
import { SystemStatusPanel } from './SystemStatusPanel';
export function SpiritProfilePanel({ activeDetail, collapsed, systemStatuses, styles, activeStyle, isSyncing, onSyncStyles, onSelectStyle, onToggleCollapsed, onOpenSettings, onOpenBackgroundGallery, isTraining, trainingSummary, trainingError, onTrainPersona, }: SpiritProfilePanelProps) {
    const assets = activeDetail ? getSpiritVisualAssets(activeDetail) : null;
    const tone = activeDetail ? getRaceTone(activeDetail.race) : 'tone-neutral';
    const choices = createTalkChoices(activeDetail);
    return (<aside className={`ever-profile ${tone} ${collapsed ? 'is-collapsed' : ''}`}>
      <div className="ever-profile__toolbar">
        <button className="ever-profile__toggle" type="button" aria-label="배경 갤러리 열기" onClick={onOpenBackgroundGallery}>
          <Images aria-hidden="true" size={20}/>
        </button>
        <button className="ever-profile__toggle" type="button" aria-label="설정 열기" onClick={onOpenSettings}>
          <Settings aria-hidden="true" size={20}/>
        </button>
        <button className="ever-profile__toggle" type="button" aria-label={collapsed ? '우측 패널 펼치기' : '우측 패널 접기'} onClick={onToggleCollapsed}>
          {collapsed ? <PanelRightOpen aria-hidden="true" size={20}/> : <PanelRightClose aria-hidden="true" size={20}/>}
        </button>
      </div>
      {collapsed ? null : (<>
      <SystemStatusPanel statuses={systemStatuses}/>
      {activeDetail ? (<>
          <section className="ever-profile-card">
            <p className="ever-kicker">BOND STATUS</p>
            <h2>{activeDetail.name}</h2>
            <span>{activeDetail.name_en}</span>
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
              {choices.map((choice) => (<div key={choice.id}>
                  <span>{choice.source}</span>
                  <strong>{choice.label}</strong>
                </div>))}
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
            <h3>정령 LoRA 학습(개조학습)</h3>
            <p>
              이 정령이 지금까지 나눈 모든 대화로 실제 역전파 기반 LoRA 학습을 실행합니다.
              CPU에서 수 분~수 시간이 걸릴 수 있으며, 최초 실행 시 기반 모델을 내려받습니다.
            </p>
            {trainingSummary && trainingSummary.persona_id === activeDetail.id && (<div className="ever-settings-result">
                <strong>학습 완료</strong>
                <span>예시 {trainingSummary.examples_used}건</span>
                <span>스텝 {trainingSummary.steps}</span>
                <span>최종 손실 {trainingSummary.final_loss.toFixed(4)}</span>
              </div>)}
            {trainingError && (<div className="ever-roster__error">
                <strong>학습 실패</strong>
                <span>{trainingError}</span>
              </div>)}
            <button className="ever-settings-reset-button" type="button" disabled={isTraining} onClick={onTrainPersona}>
              <BrainCircuit aria-hidden="true" size={16}/>
              {isTraining ? '학습 진행 중...' : '이 정령 학습 시작'}
            </button>
          </section>

          <section className="ever-panel-section">
            <h3>말투 스타일</h3>
            {activeStyle && (<div className="ever-active-style">
                <strong>{activeStyle.name}</strong>
                <span>{activeStyle.tone} / {activeStyle.formality}</span>
              </div>)}
            {styles.length === 0 ? (<button className="ever-sync-button" type="button" disabled={isSyncing} onClick={onSyncStyles}>
                {isSyncing ? '동기화 중' : '서버 스타일 동기화'}
              </button>) : (<div className="ever-style-list">
                {styles.map((style) => (<button key={style.id} type="button" className={style.is_active ? 'is-active' : ''} onClick={() => onSelectStyle(style.id)}>
                    {style.name}
                  </button>))}
              </div>)}
          </section>
        </>) : (<div className="ever-empty-panel">정령을 선택하면 TBL 기반 프로필과 원본 자산 연결 상태가 표시됩니다.</div>)}
        </>)}
    </aside>);
}
