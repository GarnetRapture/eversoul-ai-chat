import { getRaceTone, getSpiritVisualAssets } from '../../persona';
import { createBondProgress, createConversationSummary, createTalkChoices } from '../logic';
import type { ChatStageProps } from '../types';
import { LoadableAssetImage } from './LoadableAssetImage';

export function ChatStage({
  activeDetail,
  activeRoom,
  llmStatus,
  messages,
  inputText,
  isTyping,
  onInputChange,
  onSendMessage,
  messageEndRef,
}: ChatStageProps) {
  const assets = activeDetail ? getSpiritVisualAssets(activeDetail) : null;
  const tone = activeDetail ? getRaceTone(activeDetail.race) : 'tone-neutral';
  const bond = createBondProgress(activeDetail);
  const choices = createTalkChoices(activeDetail);
  const summary = createConversationSummary(activeDetail);
  const canUseComposer = Boolean(activeDetail && llmStatus?.is_loaded && !isTyping);

  return (
    <main className={`ever-stage ${tone}`}>
      {assets && <img className="ever-stage__background" src={assets.background} alt="" />}
      <div className="ever-stage__shade" />
      <header className="ever-stage__header">
        <div>
          <p className="ever-kicker">EVER TALK LOCAL LINK</p>
          <h2>{activeDetail?.name ?? '정령 선택 대기'}</h2>
        </div>
        <div className={`ever-engine ${llmStatus?.is_loaded ? 'is-on' : 'is-off'}`}>
          <span />
          {llmStatus?.is_loaded ? 'CPU LLM 연결됨' : '로컬 모델 대기'}
        </div>
      </header>

      <section className="ever-stage__body">
        <div className="ever-character">
          <LoadableAssetImage
            candidates={assets?.portraitCandidates ?? []}
            alt={activeDetail?.name ?? ''}
            className="ever-character__portrait"
            fallback={<div className="ever-character__fallback">{activeDetail?.name.charAt(0) ?? 'E'}</div>}
          />
          {activeDetail && (
            <div className="ever-character__caption">
              <strong>{activeDetail.name_en}</strong>
              <span>인연 {bond.level}Lv · {bond.current} / {bond.max}</span>
              <div className={`ever-bond-meter ${bond.meterClass}`}>
                <span />
              </div>
            </div>
          )}
        </div>

        <div className="ever-chat-panel">
          <div className="ever-chat-panel__room">
            <strong>{activeRoom?.title ?? '인연 채널'}</strong>
            <span>{summary}</span>
          </div>
          <div className="ever-messages">
            {messages.map((message) => {
              const fromUser = message.role === 'user';
              return (
                <div key={message.id} className={`ever-message ${fromUser ? 'is-user' : 'is-spirit'}`}>
                  <div className="ever-message__name">{fromUser ? '구원자' : activeDetail?.name}</div>
                  <div className="ever-message__bubble">{message.content}</div>
                </div>
              );
            })}
            {isTyping && (
              <div className="ever-message is-spirit">
                <div className="ever-message__name">{activeDetail?.name}</div>
                <div className="ever-message__bubble">
                  <span className="ever-typing"><i /><i /><i /></span>
                </div>
              </div>
            )}
            <div ref={messageEndRef} />
          </div>
          <form className="ever-composer" onSubmit={onSendMessage}>
            {choices.length > 0 && (
              <div className="ever-choice-strip">
                {choices.map((choice) => (
                  <button
                    key={choice.id}
                    className={choice.rarity === 'rare' ? 'is-rare' : ''}
                    type="button"
                    onClick={() => onInputChange(choice.label)}
                  >
                    <span>{choice.detail}</span>
                    <strong>{choice.label}</strong>
                    <small>{choice.reward}</small>
                  </button>
                ))}
              </div>
            )}
            <input
              value={inputText}
              onChange={(event) => onInputChange(event.target.value)}
              disabled={!canUseComposer}
              placeholder={activeDetail && llmStatus?.is_loaded ? `${activeDetail.name}에게 에버톡 보내기` : '정령과 로컬 모델 연결을 확인하세요'}
            />
            <button type="submit" disabled={!canUseComposer || !inputText.trim()}>
              전송
            </button>
          </form>
        </div>
      </section>
    </main>
  );
}
