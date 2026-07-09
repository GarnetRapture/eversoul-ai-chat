import { useEffect, useState } from 'react';
import type { ChangeEvent } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { Box, PlugZap, RotateCcw, Trash2, X } from 'lucide-react';
import type { AppLanguage } from '../../../shared/types';
import type { SettingsPanelProps } from '../types';

type ExternalProviderId = 'openai' | 'gemini' | 'vertex' | 'openrouter' | 'deepseek' | 'custom';

type ExternalModelOption = {
    label: string;
    value: string;
};

type ExternalProviderPreset = {
    id: ExternalProviderId;
    label: string;
    baseUrl: string;
    apiKeyHint: string;
    models: ExternalModelOption[];
};

const VERTEX_PROJECT_PLACEHOLDER = 'YOUR_PROJECT_ID';
const VERTEX_DEFAULT_LOCATION = 'global';

const EXTERNAL_PROVIDER_PRESETS: ExternalProviderPreset[] = [
    {
        id: 'openai',
        label: 'OpenAI',
        baseUrl: 'https://api.openai.com/v1',
        apiKeyHint: 'sk-...',
        models: [
            { label: 'GPT-4o mini', value: 'gpt-4o-mini' },
            { label: 'GPT-4.1 mini', value: 'gpt-4.1-mini' },
            { label: 'GPT-4.1', value: 'gpt-4.1' },
        ],
    },
    {
        id: 'gemini',
        label: 'Google Gemini',
        baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
        apiKeyHint: 'AIza...',
        models: [
            { label: 'Gemini 1.5 Flash', value: 'gemini-1.5-flash' },
            { label: 'Gemini 1.5 Pro', value: 'gemini-1.5-pro' },
            { label: 'Gemini 2.0 Flash', value: 'gemini-2.0-flash-exp' },
        ],
    },
    {
        id: 'vertex',
        label: 'Google Vertex AI',
        baseUrl: 'https://aiplatform.googleapis.com/v1/.../openapi',
        apiKeyHint: 'ya29....',
        models: [
            { label: 'Gemini 1.5 Flash', value: 'google/gemini-1.5-flash' },
            { label: 'Gemini 1.5 Pro', value: 'google/gemini-1.5-pro' },
        ],
    },
    {
        id: 'openrouter',
        label: 'OpenRouter',
        baseUrl: 'https://openrouter.ai/api/v1',
        apiKeyHint: 'sk-or-v1-...',
        models: [
            { label: 'Llama 3.1 8B Instruct', value: 'meta-llama/llama-3.1-8b-instruct' },
            { label: 'Llama 3.1 70B Instruct', value: 'meta-llama/llama-3.1-70b-instruct' },
            { label: 'Mixtral 8x7B', value: 'mistralai/mixtral-8x7b-instruct' },
        ],
    },
    {
        id: 'deepseek',
        label: 'DeepSeek',
        baseUrl: 'https://api.deepseek.com',
        apiKeyHint: 'sk-...',
        models: [
            { label: 'DeepSeek Chat', value: 'deepseek-chat' },
            { label: 'DeepSeek Reasoner', value: 'deepseek-reasoner' },
        ],
    },
    {
        id: 'custom',
        label: 'Custom OpenAI-compatible',
        baseUrl: '',
        apiKeyHint: 'API key',
        models: [],
    },
];

function buildVertexBaseUrl(projectId: string, location: string): string {
    const project = projectId.trim() || VERTEX_PROJECT_PLACEHOLDER;
    const selectedLocation = location.trim() || VERTEX_DEFAULT_LOCATION;
    return `https://aiplatform.googleapis.com/v1/projects/${project}/locations/${selectedLocation}/endpoints/openapi`;
}

function inferProvider(baseUrl: string, model: string): ExternalProviderId {
    if (baseUrl.includes('generativelanguage.googleapis.com')) {
        return 'gemini';
    }
    if (baseUrl.includes('aiplatform.googleapis.com')) {
        return 'vertex';
    }
    if (baseUrl.includes('openrouter.ai')) {
        return 'openrouter';
    }
    if (baseUrl.includes('api.deepseek.com')) {
        return 'deepseek';
    }
    if (baseUrl.includes('api.openai.com') || model.startsWith('gpt-')) {
        return 'openai';
    }
    return 'custom';
}

function getProviderPreset(id: ExternalProviderId): ExternalProviderPreset {
    return EXTERNAL_PROVIDER_PRESETS.find((preset) => preset.id === id) ?? EXTERNAL_PROVIDER_PRESETS[0];
}

function extractVertexConfig(baseUrl: string): { projectId: string; location: string } {
    const match = baseUrl.match(/\/projects\/([^/]+)\/locations\/([^/]+)\/endpoints\/openapi/);
    return {
        projectId: match?.[1] ?? '',
        location: match?.[2] ?? VERTEX_DEFAULT_LOCATION,
    };
}

export function SettingsPanel({ open: isOpen, settings, modelValidation, availableModels, selectedLocalModel, llmSessionStatuses, llmRequestStatuses, isResetting, resetSummary, resetError, importedModules, moduleImportError, labels, onClose, onReset, onSetLanguage, onSetShowReasoning, onSetInferenceMode, onSetApiProvider, onSetApiKey, onSetLocalModel, onSetExternalApiConfig, onTestExternalApi, onImportModule, onSetModuleEnabled, onDeleteModule }: SettingsPanelProps) {
    const [confirming, setConfirming] = useState(false);
    const [externalEnabled, setExternalEnabled] = useState(false);
    const [externalProvider, setExternalProvider] = useState<ExternalProviderId>('openai');
    const [externalBaseUrl, setExternalBaseUrl] = useState('https://api.openai.com/v1');
    const [externalApiKey, setExternalApiKey] = useState('');
    const [externalModel, setExternalModel] = useState('gpt-4o-mini');
    const [vertexProjectId, setVertexProjectId] = useState('');
    const [vertexLocation, setVertexLocation] = useState(VERTEX_DEFAULT_LOCATION);
    const [externalBusy, setExternalBusy] = useState(false);
    const [externalResult, setExternalResult] = useState<string | null>(null);
    const [moduleBusy, setModuleBusy] = useState(false);
    const [moduleResult, setModuleResult] = useState<string | null>(null);
    useEffect(() => {
        if (!settings) {
            return;
        }
        setExternalEnabled(settings.external_api.enabled);
        setExternalBaseUrl(settings.external_api.base_url);
        setExternalModel(settings.external_api.model);
        const provider = inferProvider(settings.external_api.base_url, settings.external_api.model);
        setExternalProvider(provider);
        const vertexConfig = extractVertexConfig(settings.external_api.base_url);
        setVertexProjectId(vertexConfig.projectId === VERTEX_PROJECT_PLACEHOLDER ? '' : vertexConfig.projectId);
        setVertexLocation(vertexConfig.location);
        setExternalApiKey('');
        setExternalResult(settings.external_api.api_key_configured ? 'API key saved.' : null);
    }, [settings]);
    if (!isOpen) {
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
    function handleExternalProviderChange(event: ChangeEvent<HTMLSelectElement>) {
        const provider = event.target.value as ExternalProviderId;
        const preset = getProviderPreset(provider);
        setExternalProvider(provider);
        if (provider === 'custom') {
            return;
        }
        if (provider === 'vertex') {
            setExternalBaseUrl(buildVertexBaseUrl(vertexProjectId, vertexLocation));
        }
        else {
            setExternalBaseUrl(preset.baseUrl);
        }
        setExternalModel(preset.models[0]?.value ?? '');
        setExternalResult(null);
    }
    function handleExternalModelChange(event: ChangeEvent<HTMLSelectElement>) {
        const value = event.target.value;
        if (value === '__custom__') {
            setExternalProvider('custom');
            return;
        }
        setExternalModel(value);
        setExternalResult(null);
    }
    function handleVertexProjectChange(event: ChangeEvent<HTMLInputElement>) {
        const value = event.target.value.trim();
        setVertexProjectId(value);
        setExternalBaseUrl(buildVertexBaseUrl(value, vertexLocation));
        setExternalResult(null);
    }
    function handleVertexLocationChange(event: ChangeEvent<HTMLInputElement>) {
        const value = event.target.value.trim();
        setVertexLocation(value || VERTEX_DEFAULT_LOCATION);
        setExternalBaseUrl(buildVertexBaseUrl(vertexProjectId, value));
        setExternalResult(null);
    }
    function getExternalPayload() {
        const baseUrl = externalBaseUrl.trim().replace(/\/chat\/completions\/?$/, '').replace(/\/$/, '');
        if (!baseUrl.startsWith('https://') && !baseUrl.startsWith('http://')) {
            throw new Error('Base URL must start with https:// or http://');
        }
        if (externalProvider === 'vertex' && !vertexProjectId.trim()) {
            throw new Error('Vertex AI Project ID is required.');
        }
        return {
            enabled: externalEnabled,
            base_url: baseUrl,
            api_key: externalApiKey.trim(),
            model: externalModel.trim(),
        };
    }
    async function handleExternalSave() {
        setExternalBusy(true);
        setExternalResult(null);
        try {
            await onSetExternalApiConfig(getExternalPayload());
            setExternalApiKey('');
            setExternalResult('Saved.');
        }
        catch (err) {
            setExternalResult(err instanceof Error ? err.message : String(err));
        }
        finally {
            setExternalBusy(false);
        }
    }
    async function handleExternalTest() {
        setExternalBusy(true);
        setExternalResult(null);
        try {
            await onSetExternalApiConfig(getExternalPayload());
            setExternalApiKey('');
            const result = await onTestExternalApi();
            setExternalResult(`${result.ok ? 'OK' : 'Failed'}: ${result.message}`);
        }
        catch (err) {
            setExternalResult(err instanceof Error ? err.message : String(err));
        }
        finally {
            setExternalBusy(false);
        }
    }
    async function handleModuleImport() {
        setModuleBusy(true);
        setModuleResult(null);
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'Risu Module', extensions: ['risum'] }],
            });
            if (typeof selected !== 'string') {
                return;
            }
            await onImportModule(selected);
            setModuleResult('Module imported.');
        }
        catch (err) {
            setModuleResult(err instanceof Error ? err.message : String(err));
        }
        finally {
            setModuleBusy(false);
        }
    }
    const externalPreset = getProviderPreset(externalProvider);
    const externalModelOptions = externalPreset.models;
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
          {settings?.inference_mode === 'local' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1rem', textAlign: 'left', marginBottom: '1rem' }}>
                  <label>
                      <span style={{ display: 'block', fontSize: '0.9rem', marginBottom: '0.2rem' }}>로컬 모델 선택</span>
                      <select 
                          value={selectedLocalModel ?? ''} 
                          onChange={(e) => void onSetLocalModel(e.target.value)}
                          style={{ padding: '0.5rem', width: '100%', background: 'rgba(0,0,0,0.2)', color: 'white', border: '1px solid rgba(255,255,255,0.1)' }}
                      >
                          {availableModels?.map(model => (
                              <option key={model.id} value={model.id}>
                                  {model.name} {model.is_downloaded ? '(설치됨)' : '(다운로드 필요)'}
                              </option>
                          ))}
                      </select>
                  </label>
              </div>
          )}
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

          <h3>외부 AI API</h3>
          <label className="ever-settings-language" style={{ marginTop: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>채팅 응답에 외부 API 사용</span>
            <input
              type="checkbox"
              checked={externalEnabled}
              onChange={(event) => setExternalEnabled(event.target.checked)}
              style={{ width: 'auto' }}
            />
          </label>
          <label className="ever-settings-language">
            <span>Provider</span>
            <select value={externalProvider} onChange={handleExternalProviderChange}>
              {EXTERNAL_PROVIDER_PRESETS.map((preset) => (
                <option key={preset.id} value={preset.id}>{preset.label}</option>
              ))}
            </select>
          </label>
          {externalProvider === 'vertex' && (
            <div className="ever-external-api-grid">
              <label className="ever-settings-language">
                <span>Vertex Project ID</span>
                <input
                  type="text"
                  value={vertexProjectId}
                  placeholder="my-gcp-project"
                  onChange={handleVertexProjectChange}
                />
              </label>
              <label className="ever-settings-language">
                <span>Vertex Location</span>
                <input
                  type="text"
                  value={vertexLocation}
                  placeholder="global"
                  onChange={handleVertexLocationChange}
                />
              </label>
            </div>
          )}
          {externalModelOptions.length > 0 && (
            <label className="ever-settings-language">
              <span>Model Preset</span>
              <select value={externalModelOptions.some((option) => option.value === externalModel) ? externalModel : '__custom__'} onChange={handleExternalModelChange}>
                {externalModelOptions.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
                <option value="__custom__">Custom model...</option>
              </select>
            </label>
          )}
          <label className="ever-settings-language">
            <span>Base URL</span>
            <input
              type="url"
              value={externalBaseUrl}
              placeholder={externalPreset.baseUrl || 'https://api.example.com/v1'}
              onChange={(event) => {
                setExternalProvider('custom');
                setExternalBaseUrl(event.target.value);
                setExternalResult(null);
              }}
            />
          </label>
          <label className="ever-settings-language">
            <span>Model</span>
            <input
              type="text"
              value={externalModel}
              placeholder={externalModelOptions[0]?.value ?? 'model-name'}
              onChange={(event) => {
                setExternalModel(event.target.value);
                setExternalResult(null);
              }}
            />
          </label>
          <label className="ever-settings-language">
            <span>API Key {settings?.external_api.api_key_configured ? '(saved)' : ''}</span>
            <input
              type="password"
              value={externalApiKey}
              placeholder={settings?.external_api.api_key_configured ? 'Leave blank to keep saved key' : externalPreset.apiKeyHint}
              onChange={(event) => setExternalApiKey(event.target.value)}
            />
          </label>
          <div className="ever-settings-result">
            <span>{externalProvider === 'vertex' ? 'Vertex AI uses an OAuth token from gcloud auth print-access-token.' : 'Select a provider and model, then paste the provider API key.'}</span>
          </div>
          {externalResult && (<div className="ever-settings-result">
              <span>{externalResult}</span>
            </div>)}
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            <button type="button" className="ever-settings-reset-button" disabled={externalBusy} onClick={handleExternalSave}>
              <PlugZap aria-hidden="true" size={16}/>
              {externalBusy ? 'Saving...' : 'Save API Settings'}
            </button>
            <button type="button" className="ever-settings-reset-button" disabled={externalBusy} onClick={handleExternalTest}>
              <PlugZap aria-hidden="true" size={16}/>
              Test Connection
            </button>
          </div>
        </section>

        <section className="ever-panel-section">
          <h3>Risu 모듈</h3>
          <div className="ever-settings-result">
            <span>활성화된 모듈의 설명과 로어북 내용이 채팅 시스템 프롬프트에 추가됩니다.</span>
          </div>
          {importedModules.length === 0 ? (<div className="ever-settings-result">
              <span>{labels.notConfigured}</span>
            </div>) : (<div className="ever-module-list">
              {importedModules.map((module) => (<div className="ever-module-item" key={module.id}>
                  <label className="ever-module-item__main">
                    <input
                      type="checkbox"
                      checked={module.enabled}
                      onChange={(event) => void onSetModuleEnabled(module.id, event.target.checked)}
                    />
                    <span>
                      <strong>{module.name}</strong>
                      <small>{module.description || module.source_path || labels.notConfigured}</small>
                      <small>lorebook {module.lorebook_count} · regex {module.regex_count} · trigger {module.trigger_count}</small>
                    </span>
                  </label>
                  <button type="button" aria-label="Delete module" onClick={() => void onDeleteModule(module.id)}>
                    <Trash2 aria-hidden="true" size={16}/>
                  </button>
                </div>))}
            </div>)}
          {(moduleResult || moduleImportError) && (<div className="ever-settings-result">
              <span>{moduleResult ?? moduleImportError}</span>
            </div>)}
          <button type="button" className="ever-settings-reset-button" disabled={moduleBusy} onClick={handleModuleImport}>
            <Box aria-hidden="true" size={16}/>
            {moduleBusy ? 'Importing...' : 'Import .risum Module'}
          </button>

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
