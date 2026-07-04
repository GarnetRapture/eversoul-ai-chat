import { memo, useMemo } from 'react';
import { GalleryHorizontal, Images, MessageCircle, Send, Sparkles, X } from 'lucide-react';
import { ASSET_ROOT, getRaceTone, getSpiritVisualAssets } from '../../persona';
import { createConversationSummary, createTalkChoices, pickRandomSpeechLine } from '../logic';
import { BACKGROUND_ASSET_FILES } from '../backgroundAssets';
import type { ChatMessage } from '../../chat';
import type { SpiritVisualAssets } from '../../persona';
import type { ChatStageProps } from '../types';
import { LoadableAssetImage } from './LoadableAssetImage';
interface ChatMessageBubbleProps {
    message: ChatMessage;
    avatarCandidates: string[];
    spiritName: string;
}
const ChatMessageBubble = memo(function ChatMessageBubble({ message, avatarCandidates, spiritName, }: ChatMessageBubbleProps) {
    const fromUser = message.role === 'user';
    return (<div className={`ever-message ${fromUser ? 'is-user' : 'is-spirit'}`}>
      {!fromUser && (<div className="ever-message__avatar">
          <LoadableAssetImage candidates={avatarCandidates} alt={spiritName} fallback={<span>{spiritName.charAt(0) || 'E'}</span>}/>
        </div>)}
      <div className="ever-message__bubble">{message.content}</div>
    </div>);
});
export function ChatStage({ activeDetail, activeRoom, llmStatus, messages, inputText, isTyping, activeStageTab, onInputChange, onSendMessage, onStageTabChange, messageEndRef, }: ChatStageProps) {
    const assets: SpiritVisualAssets | null = useMemo(() => (activeDetail ? getSpiritVisualAssets(activeDetail) : null), [activeDetail]);
    const tone = useMemo(() => (activeDetail ? getRaceTone(activeDetail.race) : 'tone-neutral'), [activeDetail]);
    const choices = useMemo(() => createTalkChoices(activeDetail), [activeDetail]);
    const summary = useMemo(() => createConversationSummary(activeDetail), [activeDetail]);
    const galleryCandidates = useMemo(() => (assets ? [...assets.avatarCandidates, ...assets.portraitCandidates] : []), [assets]);
    const speechLine = useMemo(() => pickRandomSpeechLine(activeDetail), [activeDetail?.id]);
    const canUseComposer = Boolean(activeDetail && llmStatus?.is_loaded && !isTyping);
    return (<main className={`ever-stage ${tone}`}>
      {assets && <img className="ever-stage__background" src={assets.background} alt=""/>}
      <div className="ever-stage__shade"/>
      <header className="ever-stage__header">
        <div>
          <Sparkles aria-hidden="true" size={22}/>
          <h2>{activeDetail?.name ?? '정령 선택'}</h2>
        </div>
        <X aria-hidden="true" size={20}/>
        <div className={`ever-engine ${llmStatus?.is_loaded ? 'is-on' : 'is-off'}`}>
          <span />
          {llmStatus?.is_loaded ? 'CPU LLM 연결됨' : '로컬 모델 대기'}
        </div>
      </header>

      <section className="ever-stage__body">
        <div className="ever-character" key={activeDetail?.id ?? 'none'}>
          <div className="ever-character__figure">
            {speechLine && <div className="ever-character__speech">{speechLine}</div>}
            <LoadableAssetImage candidates={assets?.portraitCandidates ?? []} alt={activeDetail?.name ?? ''} className="ever-character__portrait" fallback={<div className="ever-character__fallback">{activeDetail?.name.charAt(0) ?? 'E'}</div>}/>
          </div>
          {activeDetail && (<div className="ever-character__caption">
              <strong>{activeDetail.name_en}</strong>
              <span>{activeDetail.profile.nick_name ?? activeDetail.race}</span>
            </div>)}
        </div>

        {activeStageTab === 'chat' ? (<div className="ever-chat-panel">
            <div className="ever-chat-panel__room">
              <strong>{activeRoom?.title ?? activeDetail?.name ?? '인연 채널'}</strong>
              <span>{summary}</span>
            </div>
            <div className="ever-messages">
              {messages.length === 0 && (<div className="ever-messages__empty">
                  <strong>저장된 대화가 없습니다</strong>
                  <span>첫 메시지를 보내면 SQLite 세션에 대화가 누적됩니다.</span>
                </div>)}
              {messages.map((message) => (<ChatMessageBubble key={message.id} message={message} avatarCandidates={assets?.avatarCandidates ?? []} spiritName={activeDetail?.name ?? ''}/>))}
              {isTyping && (<div className="ever-message is-spirit">
                  <div className="ever-message__avatar">
                    <LoadableAssetImage candidates={assets?.avatarCandidates ?? []} alt={activeDetail?.name ?? ''} fallback={<span>{activeDetail?.name.charAt(0) ?? 'E'}</span>}/>
                  </div>
                  <div className="ever-message__bubble">
                    <span className="ever-typing"><i /><i /><i /></span>
                  </div>
                </div>)}
              <div ref={messageEndRef}/>
            </div>
            <form className="ever-composer" onSubmit={onSendMessage}>
              {choices.length > 0 && (<div className="ever-choice-strip">
                  {choices.map((choice) => (<button key={choice.id} type="button" onClick={() => onInputChange(choice.label)}>
                      <span>{choice.source}</span>
                      <strong>{choice.label}</strong>
                    </button>))}
                </div>)}
              <input value={inputText} onChange={(event) => onInputChange(event.target.value)} disabled={!canUseComposer} placeholder={activeDetail && llmStatus?.is_loaded ? `${activeDetail.name}에게 메시지를 입력하세요...` : '정령과 로컬 모델 연결을 확인하세요'}/>
              <button type="submit" aria-label="전송" disabled={!canUseComposer || !inputText.trim()}>
                <Send aria-hidden="true" size={22}/>
              </button>
            </form>
          </div>) : activeStageTab === 'gallery' ? (<div className="ever-gallery-panel">
            <div className="ever-chat-panel__room">
              <strong>{activeDetail?.name ?? '정령'} 이미지 갤러리</strong>
              <span>{assets?.assetFolder ?? ''}</span>
            </div>
            <div className="ever-gallery-grid">
              {galleryCandidates.map((candidate) => (<LoadableAssetImage key={candidate} candidates={[candidate]} alt={activeDetail?.name ?? ''} className="ever-gallery-image" fallback={<span>{candidate.split('/').slice(-1)[0]}</span>}/>))}
            </div>
          </div>) : (<div className="ever-gallery-panel">
            <div className="ever-chat-panel__room">
              <strong>배경 갤러리</strong>
              <span>{BACKGROUND_ASSET_FILES.length}개 배경</span>
            </div>
            <div className="ever-gallery-grid">
              {BACKGROUND_ASSET_FILES.map((file) => (<img key={file} className="ever-gallery-image" src={`${ASSET_ROOT}/backgrounds/talk/${file}`} alt={file} loading="lazy"/>))}
            </div>
          </div>)}
      </section>
      <nav className="ever-tabbar ever-stage-tabs" aria-label="에버톡 보기">
        <button className={activeStageTab === 'chat' ? 'is-active' : ''} type="button" onClick={() => onStageTabChange('chat')}>
          <MessageCircle aria-hidden="true" size={25}/>
          <span>대화</span>
        </button>
        <button className={activeStageTab === 'gallery' ? 'is-active' : ''} type="button" onClick={() => onStageTabChange('gallery')}>
          <Images aria-hidden="true" size={25}/>
          <span>갤러리</span>
        </button>
        <button className={activeStageTab === 'backgrounds' ? 'is-active' : ''} type="button" onClick={() => onStageTabChange('backgrounds')}>
          <GalleryHorizontal aria-hidden="true" size={25}/>
          <span>배경</span>
        </button>
      </nav>
    </main>);
}
