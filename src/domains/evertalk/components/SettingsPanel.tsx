import { useState } from 'react';
import type { ChangeEvent } from 'react';
import { RotateCcw, X } from 'lucide-react';
import type { AppLanguage } from '../../../shared/types';
import type { SettingsPanelProps } from '../types';
export function SettingsPanel({ open, settings, modelValidation, llmSessionStatuses, llmRequestStatuses, isResetting, resetSummary, resetError, labels, onClose, onReset, onSetLanguage, onSetShowReasoning, onSetInferenceMode, onSetApiProvider, onSetApiKey }: SettingsPanelProps) {
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
    function handleLanguageChange(event: ChangeEvent<HTMLSelectElement>) {
        void onSetLanguage(event.target.value as AppLanguage);
    }
    return (<div className="ever-settings-overlay" role="dialog" aria-modal="true">
      <div className="ever-settings-modal" style={{ width: '800px', maxWidth: '90vw' }}>
        <header className="ever-settings-modal__header">
          <h2>{labels.settings}</h2>
          <button type="button" aria-label={labels.close} onClick={handleClose}>
            <X aria-hidden="true" size={20}/>
          </button>
        </header>

        <section className="ever-panel-section">
          <h3>{labels.currentSettings}</h3>
          <div className="ever-profile-grid">
            <div>
              <small>{labels.defaultSpirit}</small>
              <strong>{settings?.default_persona_id ?? labels.notConfigured}</strong>
            </div>
            <div>
              <small>{labels.activeStyle}</small>
              <strong>{settings?.active_style_id ?? labels.notConfigured}</strong>
            </div>
            <div>
              <small>{labels.language}</small>
              <strong>{settings?.language ?? 'ko'}</strong>
            </div>
          </div>
          <label className="ever-settings-language">
            <span>{labels.displayResponseLanguage}</span>
            <select value={settings?.language ?? 'ko'} onChange={handleLanguageChange}>
              <option value="ko">{labels.languageKo}</option>
              <option value="en">{labels.languageEn}</option>
              <option value="zh_cn">{labels.languageZhCn}</option>
            </select>
          </label>
          <label className="ever-settings-language" style={{ marginTop: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>{labels.showReasoning}</span>
            <input 
              type="checkbox" 
              checked={settings?.show_reasoning ?? true} 
              onChange={(e) => void onSetShowReasoning(e.target.checked)} 
              style={{ width: 'auto' }}
            />
          </label>
        </section>

        <section className="ever-panel-section">
          <h3>{labels.inferenceMode}</h3>
          <div className="ever-language-gate__options" style={{ marginBottom: '1rem' }}>
            <button
                type="button"
                className={settings?.inference_mode === 'local' ? 'is-active' : ''}
                onClick={() => void onSetInferenceMode('local')}
            >
                <strong>{labels.modeLocal}</strong>
                <span>{labels.modeLocalDescription}</span>
            </button>
            <button
                type="button"
                className={settings?.inference_mode === 'api' ? 'is-active' : ''}
                onClick={() => void onSetInferenceMode('api')}
            >
                <strong>{labels.modeExternalApi}</strong>
                <span>{labels.modeExternalApiDescription}</span>
            </button>
          </div>
          {settings?.inference_mode === 'api' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1rem', textAlign: 'left' }}>
                  <label>
                      <span style={{ display: 'block', fontSize: '0.9rem', marginBottom: '0.2rem' }}>{labels.apiProvider}</span>
                      <select 
                          value={settings.api_provider ?? 'openai'} 
                          onChange={(e) => void onSetApiProvider(e.target.value as any)}
                          style={{ padding: '0.5rem', width: '100%', background: 'rgba(0,0,0,0.2)', color: 'white', border: '1px solid rgba(255,255,255,0.1)' }}
                      >
                          <option value="openai">OpenAI (ChatGPT)</option>
                          <option value="anthropic">Anthropic (Claude)</option>
                          <option value="gemini">Google (Gemini)</option>
                      </select>
                  </label>
                  <label>
                      <span style={{ display: 'block', fontSize: '0.9rem', marginBottom: '0.2rem' }}>{labels.apiKey}</span>
                      <input 
                          type="password" 
                          value={settings.api_key ?? ''}
                          placeholder={labels.apiKeyPlaceholder}
                          onChange={(e) => void onSetApiKey(e.target.value)}
                          style={{ padding: '0.5rem', width: '100%', background: 'rgba(0,0,0,0.2)', color: 'white', border: '1px solid rgba(255,255,255,0.1)' }}
                      />
                  </label>
              </div>
          )}
        </section>

        <section className="ever-panel-section">
          <h3>로컬 LLM 코어</h3>
          <div className="ever-profile-grid">
            <div>
              <small>모델 파일</small>
              <strong>{modelValidation ? `${Math.round(modelValidation.size_bytes / 1024 / 1024)} MB` : labels.notConfigured}</strong>
            </div>
            <div>
              <small>SHA-256</small>
              <strong>{modelValidation?.sha256.slice(0, 16) ?? labels.notConfigured}</strong>
            </div>
            <div>
              <small>사이드카 해시</small>
              <strong>{modelValidation?.hash_matches_sidecar === null ? labels.notConfigured : modelValidation?.hash_matches_sidecar ? '일치' : '불일치'}</strong>
            </div>
          </div>
          <div className="ever-settings-result">
            <strong>세션 상태</strong>
            {llmSessionStatuses.length === 0 ? (<span>{labels.notConfigured}</span>) : llmSessionStatuses.map((session) => (<span key={session.persona_id}>
                {session.persona_id} · KV {session.cached_tokens} · LoRA {session.lora_adapter_mounted ? 'on' : 'off'} · 재사용 {session.last_generation?.reused_prefix_tokens ?? 0}
              </span>))}
          </div>
          <div className="ever-settings-result">
            <strong>요청 상태</strong>
            {llmRequestStatuses.length === 0 ? (<span>{labels.notConfigured}</span>) : llmRequestStatuses.map((request) => (<span key={request.request_id}>
                {request.state} · prompt {request.prompt_tokens} · gen {request.generated_tokens} · cut {request.truncated_prompt_tokens}
              </span>))}
          </div>
        </section>

        <section className="ever-panel-section ever-settings-danger">
          <h3>{labels.resetData}</h3>
          <p>
            {labels.resetDescription}
          </p>

          {resetSummary && (<div className="ever-settings-result">
              <strong>{labels.resetComplete}</strong>
              <span>{labels.resetChatRooms(resetSummary.cleared_chat_rooms)}</span>
              <span>{labels.resetMessages(resetSummary.cleared_chat_messages)}</span>
              <span>{labels.resetPersonas(resetSummary.cleared_personas)}</span>
              <span>{labels.resetStyles(resetSummary.cleared_styles)}</span>
              <span>{labels.resetKnowledgeChunks(resetSummary.cleared_knowledge_chunks)}</span>
              <span>{labels.resetLocalMemories(resetSummary.cleared_persona_memories)}</span>
            </div>)}

          {resetError && (<div className="ever-roster__error">
              <strong>{labels.resetFailed}</strong>
              <span>{resetError}</span>
            </div>)}

          <button type="button" className={`ever-settings-reset-button ${confirming ? 'is-confirming' : ''}`} disabled={isResetting} onClick={handleResetClick}>
            <RotateCcw aria-hidden="true" size={16}/>
            {isResetting ? labels.resetting : confirming ? labels.resetConfirm : labels.resetAllData}
          </button>
        </section>
      </div>
    </div>);
}
