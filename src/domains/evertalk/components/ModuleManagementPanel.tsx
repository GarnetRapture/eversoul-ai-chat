import { open } from '@tauri-apps/plugin-dialog';
import { Box, Download, Trash2, X } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { ModuleControl } from '../../modules';
import type { ModuleManagementPanelProps } from '../types';

function controlValueLabel(control: ModuleControl): string {
    if (control.kind === 'boolean') {
        return control.value === '1' ? 'ON' : 'OFF';
    }
    return control.options.find((option) => option.value === control.value)?.label ?? control.value;
}

export function ModuleManagementPanel({
    open: isOpen,
    modules,
    moduleImportError,
    onClose,
    onImportModule,
    onSetModuleEnabled,
    onUpdateModuleControls,
    onDeleteModule,
}: ModuleManagementPanelProps) {
    const [activeModuleId, setActiveModuleId] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const [result, setResult] = useState<string | null>(null);
    const activeModule = useMemo(
        () => modules.find((module) => module.id === (activeModuleId ?? modules[0]?.id)),
        [activeModuleId, modules],
    );

    if (!isOpen) {
        return null;
    }

    async function handleImport() {
        setBusy(true);
        setResult(null);
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'Risu Module', extensions: ['risum'] }],
            });
            if (typeof selected !== 'string') {
                return;
            }
            await onImportModule(selected);
            setResult('모듈을 가져왔습니다.');
        }
        catch (err) {
            setResult(err instanceof Error ? err.message : String(err));
        }
        finally {
            setBusy(false);
        }
    }

    async function updateControl(control: ModuleControl, value: string) {
        if (!activeModule) {
            return;
        }
        const controls = activeModule.controls.map((item) =>
            item.id === control.id ? { ...item, value } : item,
        );
        await onUpdateModuleControls(activeModule.id, controls);
    }

    return (
        <div className="ever-settings-overlay" role="dialog" aria-modal="true">
            <div className="ever-module-management">
                <header className="ever-settings-modal__header">
                    <h2>모듈 관리</h2>
                    <button type="button" aria-label="닫기" onClick={onClose}>
                        <X aria-hidden="true" size={20} />
                    </button>
                </header>

                <div className="ever-module-management__layout">
                    <aside className="ever-module-management__sidebar">
                        <button
                            type="button"
                            className="ever-settings-reset-button"
                            disabled={busy}
                            onClick={handleImport}
                        >
                            <Download aria-hidden="true" size={16} />
                            {busy ? '가져오는 중...' : '.risum 가져오기'}
                        </button>

                        <div className="ever-module-management__list">
                            {modules.length === 0 ? (
                                <div className="ever-settings-result">
                                    <span>가져온 모듈이 없습니다.</span>
                                </div>
                            ) : modules.map((module) => (
                                <button
                                    key={module.id}
                                    type="button"
                                    className={module.id === activeModule?.id ? 'is-active' : ''}
                                    onClick={() => setActiveModuleId(module.id)}
                                >
                                    <Box aria-hidden="true" size={16} />
                                    <span>
                                        <strong>{module.name}</strong>
                                        <small>{module.controls.length} controls · lorebook {module.lorebook_count}</small>
                                    </span>
                                </button>
                            ))}
                        </div>
                    </aside>

                    <main className="ever-module-management__content">
                        {activeModule ? (
                            <>
                                <section className="ever-module-management__hero">
                                    <div>
                                        <h3>{activeModule.name}</h3>
                                        <p>{activeModule.description || activeModule.source_path || '설명 없음'}</p>
                                    </div>
                                    <label className="ever-module-management__enabled">
                                        <input
                                            type="checkbox"
                                            checked={activeModule.enabled}
                                            onChange={(event) => void onSetModuleEnabled(activeModule.id, event.target.checked)}
                                        />
                                        <span>{activeModule.enabled ? '활성' : '비활성'}</span>
                                    </label>
                                </section>

                                <section className="ever-module-management__controls">
                                    {activeModule.controls.length === 0 ? (
                                        <div className="ever-settings-result">
                                            <span>감지된 Risu 토글이 없습니다. 모듈 자체 활성화만 사용할 수 있습니다.</span>
                                        </div>
                                    ) : activeModule.controls.map((control) => (
                                        <div className="ever-module-control" key={control.id}>
                                            <label>
                                                <span>{control.label}</span>
                                                <small>{control.id}</small>
                                            </label>
                                            {control.kind === 'boolean' && (
                                                <button
                                                    type="button"
                                                    className={control.value === '1' ? 'is-on' : ''}
                                                    onClick={() => void updateControl(control, control.value === '1' ? '0' : '1')}
                                                >
                                                    {controlValueLabel(control)}
                                                </button>
                                            )}
                                            {control.kind === 'select' && (
                                                <select
                                                    value={control.value}
                                                    onChange={(event) => void updateControl(control, event.target.value)}
                                                >
                                                    {control.options.map((option) => (
                                                        <option key={option.value} value={option.value}>{option.label}</option>
                                                    ))}
                                                </select>
                                            )}
                                            {control.kind === 'text' && (
                                                <input
                                                    type="text"
                                                    value={control.value}
                                                    onChange={(event) => void updateControl(control, event.target.value)}
                                                />
                                            )}
                                        </div>
                                    ))}
                                </section>

                                <section className="ever-module-management__meta">
                                    <span>regex {activeModule.regex_count}</span>
                                    <span>trigger {activeModule.trigger_count}</span>
                                    <span>lorebook {activeModule.lorebook_count}</span>
                                    <button type="button" onClick={() => void onDeleteModule(activeModule.id)}>
                                        <Trash2 aria-hidden="true" size={16} />
                                        삭제
                                    </button>
                                </section>
                            </>
                        ) : (
                            <div className="ever-empty-panel">왼쪽에서 모듈을 가져오세요.</div>
                        )}

                        {(result || moduleImportError) && (
                            <div className="ever-settings-result">
                                <span>{result ?? moduleImportError}</span>
                            </div>
                        )}
                    </main>
                </div>
            </div>
        </div>
    );
}
