import { getRaceTone } from '../../persona';
import { createRosterMeta } from '../logic';
import type { SpiritRosterProps } from '../types';

export function SpiritRoster({
  spirits,
  activeSpiritId,
  searchQuery,
  onSearchChange,
  onSelect,
}: SpiritRosterProps) {
  return (
    <aside className="ever-roster">
      <div className="ever-roster__top">
        <div>
          <p className="ever-kicker">EVER TALK</p>
          <h1>정령 연결</h1>
        </div>
        <span className="ever-roster__count">{spirits.length}</span>
      </div>
      <input
        className="ever-search"
        value={searchQuery}
        onChange={(event) => onSearchChange(event.target.value)}
        placeholder="정령 이름 또는 영문명"
      />
      <div className="ever-roster__list">
        {spirits.map((spirit, index) => {
          const active = activeSpiritId === spirit.id;
          const meta = createRosterMeta(spirit, index);

          return (
            <button
              key={spirit.id}
              className={`ever-spirit-row ${active ? 'is-active' : ''} ${getRaceTone(spirit.race)}`}
              type="button"
              onClick={() => onSelect(spirit)}
            >
              <span className="ever-spirit-row__icon">
                {meta.isNew && <i>NEW</i>}
                {spirit.name.charAt(0)}
              </span>
              <span className="ever-spirit-row__copy">
                <strong>{spirit.name}</strong>
                <small>{meta.preview}</small>
              </span>
              <span className="ever-spirit-row__meta">
                <b>{spirit.grade}</b>
                {meta.unreadCount > 0 && <em>{meta.unreadCount}</em>}
              </span>
            </button>
          );
        })}
      </div>
    </aside>
  );
}
