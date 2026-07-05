import { memo, useEffect, useMemo, useRef, useState } from 'react';
import { Images, MessageCircle, Send, Sparkles, X, ZoomIn } from 'lucide-react';
import { getRaceTone, getSpiritVisualAssets } from '../../persona';
import { createConversationSummary, createTalkChoices, pickRandomSpeechLine, pickPokeReactionLine } from '../logic';
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
export function ChatStage({ activeDetail, activeRoom, llmStatus, messages, inputText, isTyping, activeStageTab, onInputChange, onSendMessage, onStageTabChange, messagesListRef, labels, onOpenProfileDetail, }: ChatStageProps) {
    const assets: SpiritVisualAssets | null = useMemo(() => (activeDetail ? getSpiritVisualAssets(activeDetail) : null), [activeDetail]);
    const tone = useMemo(() => (activeDetail ? getRaceTone(activeDetail.race) : 'tone-neutral'), [activeDetail]);
    const choices = useMemo(() => createTalkChoices(activeDetail, labels), [activeDetail, labels]);
    const summary = useMemo(() => createConversationSummary(activeDetail), [activeDetail]);
    const galleryCandidates = useMemo(() => (assets ? [...assets.avatarCandidates, ...assets.portraitCandidates] : []), [assets]);
    const speechLine = useMemo(() => pickRandomSpeechLine(activeDetail), [activeDetail?.id]);
    const canUseComposer = Boolean(activeDetail && llmStatus?.is_loaded && !isTyping);
    const [poked, setPoked] = useState(false);
    const [displayLine, setDisplayLine] = useState(speechLine);
    const [zoomedImage, setZoomedImage] = useState<string | null>(null);
    const pokeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    useEffect(() => {
        setDisplayLine(speechLine);
        setPoked(false);
        return () => {
            if (pokeTimeoutRef.current) {
                clearTimeout(pokeTimeoutRef.current);
            }
        };
    }, [activeDetail?.id]);
    function handlePortraitPoke() {
        if (!activeDetail) {
            return;
        }
        setDisplayLine(pickPokeReactionLine(activeDetail, displayLine));
        setPoked(true);
        if (pokeTimeoutRef.current) {
            clearTimeout(pokeTimeoutRef.current);
        }
        pokeTimeoutRef.current = setTimeout(() => {
            setPoked(false);
        }, 1600);
    }
    return (<main className={`ever-stage ${tone}`}>
      {assets && <img className="ever-stage__background" src={assets.background} alt=""/>}
      <div className="ever-stage__shade"/>
      <header className="ever-stage__header">
        <div>
          <Sparkles aria-hidden="true" size={22}/>
          <button className="ever-stage__profile-button" type="button" disabled={!activeDetail} onClick={onOpenProfileDetail}>
            {activeDetail?.name ?? labels.selectSpirit}
          </button>
        </div>
        <X aria-hidden="true" size={20}/>
        <div className={`ever-engine ${llmStatus?.is_loaded ? 'is-on' : 'is-off'}`}>
          <span />
          {llmStatus?.is_loaded ? labels.modelReady : labels.modelWaiting}
        </div>
      </header>

      <section className="ever-stage__body">
        <div className="ever-character" key={activeDetail?.id ?? 'none'}>
          <div className={`ever-character__figure ${poked ? 'is-poked' : ''}`}>
            {displayLine && <div className="ever-character__speech">{displayLine}</div>}
            <button
              type="button"
              className="ever-character__touch-target"
              aria-label={activeDetail ? `${activeDetail.name} ${labels.spiritReaction}` : labels.spiritReaction}
              disabled={!activeDetail}
              onClick={handlePortraitPoke}
            >
              <LoadableAssetImage candidates={assets?.portraitCandidates ?? []} alt={activeDetail?.name ?? ''} className="ever-character__portrait" fallback={<div className="ever-character__fallback">{activeDetail?.name.charAt(0) ?? 'E'}</div>}/>
              <span className="ever-character__blush" aria-hidden="true" />
            </button>
          </div>
          {activeDetail && (<button type="button" className="ever-character__caption ever-character__caption--button" onClick={onOpenProfileDetail}>
              <strong>{activeDetail.name_en}</strong>
              <span>{activeDetail.profile.nick_name ?? activeDetail.race}</span>
            </button>)}
        </div>

        {activeStageTab === 'chat' ? (<div className="ever-chat-panel">
            <div className="ever-chat-panel__room">
              <strong>{activeRoom?.title ?? activeDetail?.name ?? labels.bondChannel}</strong>
              <span>{summary}</span>
            </div>
            <div className="ever-messages" ref={messagesListRef}>
              {messages.length === 0 && (<div className="ever-messages__empty">
                  <strong>{labels.noSavedMessages}</strong>
                  <span>{labels.firstMessageHint}</span>
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
            </div>
            <form className="ever-composer" onSubmit={onSendMessage}>
              {choices.length > 0 && (<div className="ever-choice-strip">
                  {choices.map((choice) => (<button key={choice.id} type="button" onClick={() => onInputChange(choice.label)}>
                      <span>{choice.source}</span>
                      <strong>{choice.label}</strong>
                    </button>))}
                </div>)}
              <input value={inputText} onChange={(event) => onInputChange(event.target.value)} disabled={!canUseComposer} placeholder={activeDetail && llmStatus?.is_loaded ? labels.messagePlaceholder(activeDetail.name) : labels.modelRequiredPlaceholder}/>
              <button type="submit" aria-label={labels.send} disabled={!canUseComposer || !inputText.trim()}>
                <Send aria-hidden="true" size={22}/>
              </button>
            </form>
          </div>) : (<div className="ever-gallery-panel">
            <div className="ever-chat-panel__room">
              <strong>{activeDetail?.name ?? labels.selectSpirit} {labels.imageGallery}</strong>
              <span>{assets?.assetFolder ?? ''}</span>
            </div>
            <div className="ever-gallery-grid">
              {galleryCandidates.map((candidate) => (<button key={candidate} type="button" className="ever-gallery-tile ever-gallery-tile--button" aria-label={`${candidate} ${labels.zoomImage}`} onClick={() => setZoomedImage(candidate)}>
                  <LoadableAssetImage candidates={[candidate]} alt={activeDetail?.name ?? ''} fallback={<span>{candidate.split('/').slice(-1)[0]}</span>}/>
                  <span className="ever-gallery-tile__zoom-hint" aria-hidden="true">
                    <ZoomIn size={18}/>
                  </span>
                </button>))}
            </div>
          </div>)}
      </section>
      <nav className="ever-tabbar ever-stage-tabs" aria-label={labels.rosterTitle}>
        <button className={activeStageTab === 'chat' ? 'is-active' : ''} type="button" onClick={() => onStageTabChange('chat')}>
          <MessageCircle aria-hidden="true" size={25}/>
          <span>{labels.chat}</span>
        </button>
        <button className={activeStageTab === 'gallery' ? 'is-active' : ''} type="button" onClick={() => onStageTabChange('gallery')}>
          <Images aria-hidden="true" size={25}/>
          <span>{labels.gallery}</span>
        </button>
      </nav>
      {zoomedImage && (<div className="ever-background-zoom-overlay" role="dialog" aria-modal="true" onClick={() => setZoomedImage(null)}>
          <button type="button" className="ever-background-zoom-close" aria-label={labels.close} onClick={() => setZoomedImage(null)}>
            <X aria-hidden="true" size={24}/>
          </button>
          <img className="ever-background-zoom-image" src={zoomedImage} alt={activeDetail?.name ?? ''} onClick={(event) => event.stopPropagation()}/>
          <span className="ever-background-zoom-caption">{zoomedImage.split('/').slice(-1)[0]}</span>
        </div>)}
    </main>);
}
