import { HeartHandshake, PanelLeftClose, PanelLeftOpen, Star, Trophy, Users } from 'lucide-react';
import { getRaceTone, getSpiritVisualAssets, parseSpiritDetail } from '../../persona';
import { createRosterMeta } from '../logic';
import type { SpiritRosterProps } from '../types';
import { LoadableAssetImage } from './LoadableAssetImage';
export function SpiritRoster({ spirits, activeSpiritId, searchQuery, loadError, defaultPersonaId, activeTab, collapsed, bondRanking, bondRankingLoading, onSearchChange, onSelect, onSetDefault, onTabChange, onToggleCollapsed, }: SpiritRosterProps) {
    const hasSpirits = spirits.length > 0;
    return (<aside className={`ever-roster ${collapsed ? 'is-collapsed' : ''}`}>
      <div className="ever-roster__top">
        {!collapsed && (<div>
            <h1>에버톡</h1>
            <span>정령 메시지 ({spirits.length})</span>
          </div>)}
        <button className="ever-roster__toggle" type="button" aria-label={collapsed ? '좌측 패널 펼치기' : '좌측 패널 접기'} onClick={onToggleCollapsed}>
          {collapsed ? <PanelLeftOpen aria-hidden="true" size={20}/> : <PanelLeftClose aria-hidden="true" size={20}/>}
        </button>
      </div>
      {collapsed ? null : (<>
      <input className="ever-search" value={searchQuery} onChange={(event) => onSearchChange(event.target.value)} placeholder="정령 이름 또는 영문명"/>
      <div className="ever-roster__list">
        {loadError && (<div className="ever-roster__error">
            <strong>정령 데이터 로드 실패</strong>
            <span>{loadError}</span>
          </div>)}
        {!loadError && spirits.length === 0 && (<div className="ever-roster__empty">
            <strong>정령 DB 수립 대기</strong>
            <span>persona pack과 SQLite 연결 상태를 확인 중입니다.</span>
          </div>)}
        {activeTab === 'list' && spirits.map((spirit) => {
                const active = activeSpiritId === spirit.id;
                const isDefault = defaultPersonaId === spirit.id;
                const meta = createRosterMeta(spirit);
                const detail = parseSpiritDetail(spirit);
                const assets = getSpiritVisualAssets(detail);
                return (<div key={spirit.id} className={`ever-spirit-row ${active ? 'is-active' : ''} ${getRaceTone(spirit.race)}`}>
                <button className="ever-spirit-row__select" type="button" onClick={() => onSelect(spirit)}>
                  <span className="ever-spirit-row__icon">
                    <LoadableAssetImage candidates={assets.avatarCandidates} alt={spirit.name} fallback={<span>{spirit.name.charAt(0)}</span>}/>
                  </span>
                  <span className="ever-spirit-row__copy">
                    <strong>{spirit.name}</strong>
                    <small>{meta.preview}</small>
                  </span>
                </button>
                <span className="ever-spirit-row__meta">
                  <b>{spirit.grade}</b>
                  <button className={isDefault ? 'is-default' : ''} type="button" aria-label={`${spirit.name} 기본 프로필 지정`} onClick={() => {
                        void onSetDefault(spirit.id);
                    }}>
                    <Star aria-hidden="true" size={16}/>
                  </button>
                </span>
              </div>);
            })}
        {activeTab === 'bondRanking' && bondRankingLoading && (<div className="ever-roster__empty">
            <strong>인연도 랭킹 조회 중</strong>
          </div>)}
        {activeTab === 'bondRanking' && !bondRankingLoading && bondRanking.length === 0 && (<div className="ever-roster__empty">
            <strong>누적된 대화가 없습니다</strong>
            <span>정령과 대화를 나누면 실제 메시지/기억 누적량을 기준으로 랭킹이 산출됩니다.</span>
          </div>)}
        {activeTab === 'bondRanking' && !bondRankingLoading && bondRanking.map((entry, index) => (<div key={entry.persona_id} className={`ever-spirit-row ${activeSpiritId === entry.persona_id ? 'is-active' : ''}`}>
            <button className="ever-spirit-row__select" type="button" onClick={() => {
                    const spirit = spirits.find((s) => s.id === entry.persona_id);
                    if (spirit) {
                        onSelect(spirit);
                    }
                }}>
              <span className="ever-spirit-row__icon">
                <span>{index + 1}</span>
              </span>
              <span className="ever-spirit-row__copy">
                <strong>{entry.name}</strong>
                <small>메시지 {entry.message_count} · 기억 {entry.memory_count}</small>
              </span>
            </button>
            <span className="ever-spirit-row__meta">
              <b>{entry.bond_score}</b>
            </span>
          </div>))}
        {activeTab === 'familiarity' && (<div className="ever-roster__empty">
            <strong>친밀도 DB 미수립</strong>
            <span>
              {hasSpirits
                    ? '현재 SQLite 스키마에 해당 누적 지표 테이블이 없어 실제 값만 표시하도록 대기합니다.'
                    : '정령 DB 로드 이후 실제 누적 지표를 연결합니다.'}
            </span>
          </div>)}
      </div>
      <nav className="ever-tabbar" aria-label="에버톡 보기">
        <button className={activeTab === 'list' ? 'is-active' : ''} type="button" onClick={() => onTabChange('list')}>
          <Users aria-hidden="true" size={22}/>
          <span>목록</span>
        </button>
        <button className={activeTab === 'bondRanking' ? 'is-active' : ''} type="button" onClick={() => onTabChange('bondRanking')}>
          <Trophy aria-hidden="true" size={22}/>
          <span>인연도 랭킹</span>
        </button>
        <button className={activeTab === 'familiarity' ? 'is-active' : ''} type="button" onClick={() => onTabChange('familiarity')}>
          <HeartHandshake aria-hidden="true" size={22}/>
          <span>친밀도</span>
        </button>
      </nav>
      </>)}
    </aside>);
}
