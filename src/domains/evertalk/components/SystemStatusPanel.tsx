import type { SystemStatusPanelProps } from '../types';
export function SystemStatusPanel({ statuses }: SystemStatusPanelProps) {
    return (<section className="ever-panel-section ever-system-status">
      <h3>로컬 실행 상태</h3>
      <div className="ever-system-status__list">
        {statuses.map((status) => (<div key={status.id} className={`ever-system-status__item is-${status.state}`}>
            <span>{status.label}</span>
            <strong>{status.detail}</strong>
          </div>))}
      </div>
    </section>);
}
